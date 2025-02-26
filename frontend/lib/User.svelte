<script lang="ts">
    import Button, { Label } from "@smui/button";
    import Textfield from "@smui/textfield";

    import api, { userInfo } from "../api.svelte";
    import { onMount } from "svelte";

    let usernameInput = $state("");
    let passwordInput = $state("");
    let error = $state("");
    const debounceTimeout = 500;

    const loginSubmit = async (event: Event) => {
        event.preventDefault();
        try {
            await api.login({ username: usernameInput, password: passwordInput });
        } catch (err: any) {
            error = err.toString();
        }
    };

    const registerSumbit = async (event: Event) => {
        event.preventDefault();
        try {
            await api.newUser({ username: usernameInput, password: passwordInput });
        } catch (err: any) {
            error = err.toString();
        }
    };

    let usernameTimer: number;
    const validateUsername = () => {
        clearTimeout(usernameTimer);
        usernameTimer = setTimeout(async () => {
            try {
                await api.validate('username', usernameInput);
                error = "";
            } catch (err: any) {
                error = err.toString();
            }
        }, debounceTimeout);
    };

    let passwordTimer: number;
    const validatePassword = () => {
        clearTimeout(passwordTimer);
        passwordTimer = setTimeout(async () => {
            try {
                await api.validate('password', passwordInput);
                error = "";
            } catch (err: any) {
                error = err.toString();
            }
        }, debounceTimeout);
    };

    onMount(api.updateAPI);
</script>

{#if !userInfo.isLoggedIn}
    <form class="login-form" onsubmit={loginSubmit}>
        <!-- Login form -->
        <div>
            <Textfield
                type="username"
                bind:value={usernameInput}
                label="Username"
                input$autocomplete="username"
                onkeyup={validateUsername}
            ></Textfield>
        </div>

        <div>
            <Textfield
                type="password"
                bind:value={passwordInput}
                label="Password"
                input$autocomplete="new-password"
                onkeyup={validatePassword}
            ></Textfield>
        </div>

        <div class="row">
            <Button variant="raised" type="submit">
                <Label>Login</Label>
            </Button>
            <Button onclick={registerSumbit}>
                <Label>Register</Label>
            </Button>
        </div>

        {#if error.length != 0}
            <p class="error">{error}</p>
        {/if}
    </form>
{:else}
    <!-- User info -->
    <div class="login-form">
        <p>Logged in as {userInfo.username}</p><br/>
        <p>Your balance: {userInfo.balance}€</p>
        <Button variant="raised" onclick={api.logout}>
            <Label>Logout</Label>
        </Button>
    </div>
{/if}

<style>
    .login-form {
        display: flex;
        flex-direction: column;
        justify-content: center;
        width: 100%;
        align-items: center;
    }
    .row {
        display: flex;
        flex-direction: row;
    }
    .error {
        color: red;
    }
</style>
