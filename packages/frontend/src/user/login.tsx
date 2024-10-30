import { type SubmitHandler, createForm } from "@modular-forms/solid";
import {
    GithubAuthProvider,
    GoogleAuthProvider,
    type User,
    createUserWithEmailAndPassword,
    getAuth,
    signInWithEmailAndPassword,
    signInWithPopup,
} from "firebase/auth";
import { useFirebaseApp } from "solid-firebase";

import { IconButton } from "../components";

import SignInIcon from "lucide-solid/icons/log-in";
import SignUpIcon from "lucide-solid/icons/user-pen";

import "./login.css";

type EmailAndPassword = {
    email: string;
    password: string;
};

export function Login(props: {
    onComplete?: (user: User) => void;
}) {
    const firebaseApp = useFirebaseApp();

    const [, { Form, Field }] = createForm<EmailAndPassword>();

    const onSubmit: SubmitHandler<EmailAndPassword> = (values, event) => {
        const submitter = event.submitter as HTMLButtonElement;
        if (submitter.value === "sign-in") {
            return signIn(values);
        } else if (submitter.value === "sign-up") {
            return signUp(values);
        } else {
            throw new Error(`Unrecognized submitter: ${submitter.value}`);
        }
    };

    const signIn = async (values: EmailAndPassword) => {
        const { email, password } = values;
        const result = await signInWithEmailAndPassword(getAuth(firebaseApp), email, password);
        props.onComplete?.(result.user);
    };

    const signUp = async (values: EmailAndPassword) => {
        console.log(values);
        const { email, password } = values;
        const result = await createUserWithEmailAndPassword(getAuth(firebaseApp), email, password);
        props.onComplete?.(result.user);
    };

    const signInWithGoogle = async () => {
        const provider = new GoogleAuthProvider();
        const result = await signInWithPopup(getAuth(firebaseApp), provider);
        props.onComplete?.(result.user);
    };

    const signInWithGitHub = async () => {
        const provider = new GithubAuthProvider();
        const result = await signInWithPopup(getAuth(firebaseApp), provider);
        props.onComplete?.(result.user);
    };

    return (
        <div class="login">
            <Form onSubmit={onSubmit}>
                <Field name="email">
                    {(field, props) => (
                        <input
                            {...props}
                            type="email"
                            required
                            value={field.value}
                            placeholder="Email address"
                        />
                    )}
                </Field>
                <Field name="password">
                    {(field, props) => (
                        <input
                            {...props}
                            type="password"
                            required
                            value={field.value}
                            placeholder="Password"
                        />
                    )}
                </Field>
                <div class="buttons">
                    <IconButton type="submit" value="sign-in">
                        <SignInIcon />
                        Login
                    </IconButton>
                    <IconButton type="submit" value="sign-up">
                        <SignUpIcon />
                        Sign up
                    </IconButton>
                </div>
            </Form>
            <div class="separator">{"Or continue with"}</div>
            <div class="provider-list">
                <IconButton onClick={signInWithGoogle} tooltip="Login with Google">
                    <img
                        height="28"
                        width="28"
                        src="https://cdn.jsdelivr.net/npm/simple-icons@latest/icons/google.svg"
                    />
                </IconButton>
                <IconButton onClick={signInWithGitHub} tooltip="Login with GitHub">
                    <img
                        height="28"
                        width="28"
                        src="https://cdn.jsdelivr.net/npm/simple-icons@latest/icons/github.svg"
                    />
                </IconButton>
            </div>
        </div>
    );
}
