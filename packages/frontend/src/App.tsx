import { Repo } from "@automerge/automerge-repo";
import { BrowserWebSocketClientAdapter } from "@automerge/automerge-repo-network-websocket";
import { IndexedDBStorageAdapter } from "@automerge/automerge-repo-storage-indexeddb";
import { initializeApp } from "firebase/app";
import invariant from "tiny-invariant";
import * as uuid from "uuid";

import { MultiProvider } from "@solid-primitives/context";
import { Navigate, type RouteDefinition, type RouteSectionProps, Router } from "@solidjs/router";
import { FirebaseProvider } from "solid-firebase";
import { Match, Switch, createResource, lazy, useContext } from "solid-js";

import type { JsonValue } from "catcolab-api";
import { RepoContext, RpcContext, createRpcClient } from "./api";
import { newModelDocument } from "./document/types";
import { HelperContainer, lazyMdx } from "./page/help_page";
import { TheoryLibraryContext, stdTheories } from "./stdlib";

const serverUrl = import.meta.env.VITE_SERVER_URL;
const repoUrl = import.meta.env.VITE_AUTOMERGE_REPO_URL;

// TODO: Move to vite env.
const firebaseConfig = {
    apiKey: "AIzaSyAsFvrzQg_V8cVhGi9PNkZiueGF0iDH9Ws",
    authDomain: "catcolab-next.firebaseapp.com",
    projectId: "catcolab-next",
    storageBucket: "catcolab-next.appspot.com",
    messagingSenderId: "666779369059",
    appId: "1:666779369059:web:f0319c1513d77996650256",
    measurementId: "G-WKFYSTDYLF",
};

const Root = (props: RouteSectionProps<unknown>) => {
    invariant(serverUrl, "Must set environment variable VITE_SERVER_URL");
    invariant(repoUrl, "Must set environment variable VITE_AUTOMERGE_REPO_URL");

    const client = createRpcClient(serverUrl);

    const repo = new Repo({
        storage: new IndexedDBStorageAdapter("catcolab"),
        network: [new BrowserWebSocketClientAdapter(repoUrl)],
    });

    const firebaseApp = initializeApp(firebaseConfig);

    return (
        <MultiProvider
            values={[
                [RpcContext, client],
                [RepoContext, repo],
                [TheoryLibraryContext, stdTheories],
            ]}
        >
            <FirebaseProvider app={firebaseApp}>{props.children}</FirebaseProvider>
        </MultiProvider>
    );
};

function CreateModel() {
    const rpc = useContext(RpcContext);
    invariant(rpc, "Missing context to create model");

    const init = newModelDocument();

    const [ref] = createResource<string>(async () => {
        const result = await rpc.new_ref.mutate(init as JsonValue);
        invariant(result.tag === "Ok", "Failed to create model");
        return result.content;
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
