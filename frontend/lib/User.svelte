<script lang="ts">
    import Button, { Label } from "@smui/button";
    import Textfield from "@smui/textfield";

    import { userInfo, login, newUser, logout, updateAPI } from "../api.svelte";
    import { onMount } from "svelte";

    let usernameInput = $state("");
    let passwordInput = $state("");
    let error = $state("");

    const loginSubmit = async (event: Event) => {
        event.preventDefault();
        try {
            await login({ username: usernameInput, password: passwordInput });
        } catch (err) {
            error = err.toString();
        }
    };

    const registerSumbit = async (event: Event) => {
        event.preventDefault();
        try {
            await newUser({ username: usernameInput, password: passwordInput });
        } catch (err) {
            error = err.toString();
        }
    };

    onMount(updateAPI);
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
            ></Textfield>
        </div>

        <div>
            <Textfield
                type="password"
                bind:value={passwordInput}
                label="Password"
                input$autocomplete="new-password"
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
        <Button variant="raised" onclick={logout}>
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
