import { Repo } from "@automerge/automerge-repo";
import { BrowserWebSocketClientAdapter } from "@automerge/automerge-repo-network-websocket";
import { IndexedDBStorageAdapter } from "@automerge/automerge-repo-storage-indexeddb";
import invariant from "tiny-invariant";
import * as uuid from "uuid";

import { MultiProvider } from "@solid-primitives/context";
import { Navigate, type RouteDefinition, type RouteSectionProps, Router } from "@solidjs/router";
import { Match, Switch, createResource, lazy, useContext } from "solid-js";

import { RPCContext, RepoContext, createRPCClient } from "./api";
import { newModelDocument } from "./document/types";
import { HelperContainer, lazyMdx } from "./page/help_page";
import { TheoryLibraryContext, stdTheories } from "./stdlib";

const serverUrl: string = import.meta.env.VITE_SERVER_URL;
const repoUrl: string = import.meta.env.VITE_AUTOMERGE_REPO_URL;

const Root = (props: RouteSectionProps<unknown>) => {
    invariant(serverUrl, "Must set environment variable VITE_SERVER_URL");
    invariant(repoUrl, "Must set environment variable VITE_AUTOMERGE_REPO_URL");

    const client = createRPCClient(serverUrl);
    const repo = new Repo({
        storage: new IndexedDBStorageAdapter("catcolab"),
        network: [new BrowserWebSocketClientAdapter(repoUrl)],
    });

    return (
        <MultiProvider
            values={[
                [RPCContext, client],
                [RepoContext, repo],
                [TheoryLibraryContext, stdTheories],
            ]}
        >
            {props.children}
        </MultiProvider>
    );
};

function CreateModel() {
    const client = useContext(RPCContext);
    invariant(client, "Missing context to create model");

    const init = newModelDocument();

    const [ref] = createResource<string>(async () => {
        return await client.mutation(["new_ref", init]);
    });

    return (
        <Switch>
            <Match when={ref.error}>
                <span>Error: {ref.error}</span>
            </Match>
            <Match when={ref()}>{(ref) => <Navigate href={`/model/${ref()}`} />}</Match>
        </Switch>
    );
}

const refIsUUIDFilter = {
    ref: (ref: string) => uuid.validate(ref),
};

const routes: RouteDefinition[] = [
    {
        path: "/",
        component: CreateModel,
    },
    {
        path: "/model/:ref",
        matchFilters: refIsUUIDFilter,
        component: lazy(() => import("./document/model_document_editor")),
    },
    {
        path: "/analysis/:ref",
        matchFilters: refIsUUIDFilter,
        component: lazy(() => import("./document/analysis_document_editor")),
    },
    {
        path: "/help",
        component: HelperContainer,
        children: [
            {
                path: "/",
                component: lazyMdx(() => import("./help/index.mdx")),
            },
            {
                path: "/credits",
                component: lazyMdx(() => import("./help/credits.mdx")),
            },
        ],
    },
];

function App() {
    return <Router root={Root}>{routes}</Router>;
}

export default App;
