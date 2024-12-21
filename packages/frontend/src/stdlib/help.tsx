import { useParams } from "@solidjs/router";
import { useContext } from "solid-js";
import invariant from "tiny-invariant";

import { TheoryHelp } from "../theory/help";
import { TheoryLibraryContext } from "./context";

/** Help page for a theory in the standard library. */
export default function TheoryHelpPage() {
    const theories = useContext(TheoryLibraryContext);
    invariant(theories, "Library of theories must be provided as context");

    const params = useParams();
    const theoryId = params.id;
    invariant(theoryId, "Theory ID must be provided as parameter");
    const theory = theories.get(theoryId);

    return <TheoryHelp theory={theory} />;
}
