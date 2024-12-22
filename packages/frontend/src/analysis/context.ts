import { type Accessor, createContext } from "solid-js";

import type { LiveAnalysisDocument } from "./document";

/** Context for a live analysis. */
export const LiveAnalysisContext = createContext<Accessor<LiveAnalysisDocument>>();
