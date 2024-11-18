import { useNavigate, useParams } from "@solidjs/router";
import { Match, Show, Switch, createResource, useContext } from "solid-js";
import invariant from "tiny-invariant";

import type { JsonValue } from "catcolab-api";
import { newAnalysisDocument } from "../analysis/document";
import { RepoContext, RpcContext, getLiveDoc } from "../api";
import { IconButton, InlineInput } from "../components";
import { newDiagramDocument } from "../diagram";
import {
    type CellConstructor,
    type FormalCellEditorProps,
    NotebookEditor,
    cellShortcutModifier,
    newFormalCell,
} from "../notebook";
import { BrandedToolbar, HelpButton } from "../page";
import { TheoryLibraryContext } from "../stdlib";
import type { ModelTypeMeta } from "../theory";
import { MaybePermissionsButton } from "../user";
import { LiveModelContext } from "./context";
import { type LiveModelDocument, type ModelDocument, enlivenModelDocument } from "./document";
import { MorphismCellEditor } from "./morphism_cell_editor";
import { ObjectCellEditor } from "./object_cell_editor";
import { TheorySelectorDialog } from "./theory_selector";
import {
    type ModelJudgment,
    type MorphismDecl,
    type ObjectDecl,
    newMorphismDecl,
    newObjectDecl,
} from "./types";

import "./model_editor.css";

import ChartSpline from "lucide-solid/icons/chart-spline";
import Network from "lucide-solid/icons/network";

export default function ModelPage() {
    const params = useParams();
    const refId = params.ref;
    invariant(refId, "Must provide model ref as parameter to model page");

    const rpc = useContext(RpcContext);
    const repo = useContext(RepoContext);
    const theories = useContext(TheoryLibraryContext);
    invariant(rpc && repo && theories, "Missing context for model page");

    const [liveModel] = createResource<LiveModelDocument>(async () => {
        const liveDoc = await getLiveDoc<ModelDocument>(rpc, repo, refId);
        return enlivenModelDocument(refId, liveDoc, theories);
    });

    return <ModelDocumentEditor liveModel={liveModel()} />;
}

export function ModelDocumentEditor(props: {
    liveModel?: LiveModelDocument;
}) {
    const rpc = useContext(RpcContext);
    invariant(rpc, "Missing context for model document editor");

    const navigate = useNavigate();

    const createDiagram = async (modelRefId: string) => {
        const init = newDiagramDocument(modelRefId);

        const result = await rpc.new_ref.mutate({
            content: init as JsonValue,
            permissions: {
                anyone: "Read",
            },
        });
        invariant(result.tag === "Ok", "Failed to create a new diagram");
        const newRef = result.content;

        navigate(`/diagram/${newRef}`);
    };

    const createAnalysis = async (modelRefId: string) => {
        const init = newAnalysisDocument(modelRefId);

        const result = await rpc.new_ref.mutate({
            content: init as JsonValue,
            permissions: {
                anyone: "Read",
            },
        });
        invariant(result.tag === "Ok", "Failed to create a new analysis");
        const newRef = result.content;

        navigate(`/analysis/${newRef}`);
    };

    return (
        <div class="growable-container">
            <BrandedToolbar>
                <HelpButton />
                <MaybePermissionsButton permissions={props.liveModel?.liveDoc.permissions} />
                <Show when={props.liveModel?.theory()?.supportsInstances}>
                    <IconButton
                        onClick={() => props.liveModel && createDiagram(props.liveModel.refId)}
                        tooltip="Create a diagram in this model"
                    >
                        <Network />
                    </IconButton>
                </Show>
                <IconButton
                    onClick={() => props.liveModel && createAnalysis(props.liveModel.refId)}
                    tooltip="Analyze this model"
                >
                    <ChartSpline />
                </IconButton>
            </BrandedToolbar>
            <Show when={props.liveModel}>
                {(liveModel) => <ModelPane liveModel={liveModel()} />}
            </Show>
        </div>
    );
}

/** Pane containing a model notebook plus a header with the title and theory.
 */
export function ModelPane(props: {
    liveModel: LiveModelDocument;
}) {
    const theories = useContext(TheoryLibraryContext);
    invariant(theories, "Library of theories should be provided as context");

    const liveDoc = () => props.liveModel.liveDoc;

    return (
        <div class="notebook-container">
            <div class="model-head">
                <div class="title">
                    <InlineInput
                        text={liveDoc().doc.name}
                        setText={(text) => {
                            liveDoc().changeDoc((doc) => {
                                doc.name = text;
                            });
                        }}
                        placeholder="Untitled"
                    />
                </div>
                <TheorySelectorDialog
                    theory={props.liveModel.theory()}
                    setTheory={(id) => {
                        liveDoc().changeDoc((model) => {
                            model.theory = id;
                        });
                    }}
                    theories={theories}
                    disabled={liveDoc().doc.notebook.cells.some((cell) => cell.tag === "formal")}
                />
            </div>
            <ModelNotebookEditor liveModel={props.liveModel} />
        </div>
    );
}

/** Notebook editor for a model of a double theory.
 */
export function ModelNotebookEditor(props: {
    liveModel: LiveModelDocument;
}) {
    const liveDoc = () => props.liveModel.liveDoc;

    return (
        <LiveModelContext.Provider value={props.liveModel}>
            <NotebookEditor
                handle={liveDoc().docHandle}
                path={["notebook"]}
                notebook={liveDoc().doc.notebook}
                changeNotebook={(f) => {
                    liveDoc().changeDoc((doc) => f(doc.notebook));
                }}
                formalCellEditor={ModelCellEditor}
                cellConstructors={modelCellConstructors(props.liveModel.theory()?.modelTypes ?? [])}
                cellLabel={judgmentLabel}
            />
        </LiveModelContext.Provider>
    );
}

/** Editor for a notebook cell in a model notebook.
 */
function ModelCellEditor(props: FormalCellEditorProps<ModelJudgment>) {
    return (
        <Switch>
            <Match when={props.content.tag === "object"}>
                <ObjectCellEditor
                    object={props.content as ObjectDecl}
                    modifyObject={(f) => props.changeContent((content) => f(content as ObjectDecl))}
                    isActive={props.isActive}
                    actions={props.actions}
                />
            </Match>
            <Match when={props.content.tag === "morphism"}>
                <MorphismCellEditor
                    morphism={props.content as MorphismDecl}
                    modifyMorphism={(f) =>
                        props.changeContent((content) => f(content as MorphismDecl))
                    }
                    isActive={props.isActive}
                    actions={props.actions}
                />
            </Match>
        </Switch>
    );
}

function modelCellConstructors(modelTypes: ModelTypeMeta[]): CellConstructor<ModelJudgment>[] {
    return modelTypes.map((meta) => {
        const { name, description, shortcut } = meta;
        return {
            name,
            description,
            shortcut: shortcut && [cellShortcutModifier, ...shortcut],
            construct() {
                return meta.tag === "ObType"
                    ? newFormalCell(newObjectDecl(meta.obType))
                    : newFormalCell(newMorphismDecl(meta.morType));
            },
        };
    });
}

function judgmentLabel(judgment: ModelJudgment): string | undefined {
    const liveModel = useContext(LiveModelContext);
    const theory = liveModel?.theory();
    if (judgment.tag === "object") {
        return theory?.modelObTypeMeta(judgment.obType)?.name;
    }
    if (judgment.tag === "morphism") {
        return theory?.modelMorTypeMeta(judgment.morType)?.name;
    }
}
