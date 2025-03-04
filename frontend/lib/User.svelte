<script lang="ts">
    import Button, { Label } from "@smui/button";
    import Textfield from "@smui/textfield";

    import api, { userInfo } from "../api.svelte";
    import { onMount } from "svelte";

    const debounceTimeout = 500;

    let usernameInput = $state("");
    let passwordInput = $state("");
    let loginError = $state("");

    let transferUserInput = $state("");
    let transferAmountInput = $state("");
    let transferError = $state("");

    const loginSubmit = async (event: Event) => {
        event.preventDefault();
        loginError = "";
        try {
            await api.login({ username: usernameInput, password: passwordInput });
        } catch (err: any) {
            loginError = err.toString();
        }
    };

    const registerSumbit = async (event: Event) => {
        event.preventDefault();
        loginError = "";
        try {
            await api.newUser({ username: usernameInput, password: passwordInput });
        } catch (err: any) {
            loginError = err.toString();
        }
    };

    const transferSubmit = async (event: Event) => {
        event.preventDefault();
        const amountCents = Math.floor(parseFloat(transferAmountInput) * 100);
        try {
            await api.transfer({ recipient: transferUserInput, amount_cents: amountCents });
            transferError = "";
        } catch (err: any) {
            transferError = err.toString();
        }
    };

    let usernameTimer: number;
    const validateUsername = () => {
        clearTimeout(usernameTimer);
        usernameTimer = setTimeout(async () => {
            try {
                await api.validate('username', usernameInput);
                loginError = "";
            } catch (err: any) {
                loginError = err.toString();
            }
        }, debounceTimeout);
    };

    let passwordTimer: number;
    const validatePassword = () => {
        clearTimeout(passwordTimer);
        passwordTimer = setTimeout(async () => {
            try {
                await api.validate('password', passwordInput);
                loginError = "";
            } catch (err: any) {
                loginError = err.toString();
            }
        }, debounceTimeout);
    };

    let transferUserTimer: number;
    const validateTransferUser = () => {
        clearTimeout(transferUserTimer);
        transferUserTimer = setTimeout(async () => {
            try {
                await api.validate('username', transferUserInput);
                transferError = "";
            } catch (err: any) {
                transferError = err.toString();
            }
        }, debounceTimeout);
    };

    let transferAmountTimer: number;
    const validateTransferAmount = () => {
        clearTimeout(transferAmountTimer);
        transferAmountTimer = setTimeout(async () => {
            try {
                await api.validate('currency', transferAmountInput);
                transferError = "";
            } catch (err: any) {
                transferError = err.toString();
            }
        }, debounceTimeout);
    };

    onMount(api.update);
</script>

{#if !userInfo.isLoggedIn}
    <form class="column" onsubmit={loginSubmit}>
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

        {#if loginError.length != 0}
            <p class="error">{loginError}</p>
        {/if}
    </form>
{:else}
    <!-- User info -->
    <div class="column">
        <h2>Profile</h2>
        <p>Logged in as {userInfo.username}</p><br/>
        <p>Your balance: {userInfo.balance}€</p>
        <Button variant="raised" onclick={api.logout}>
            <Label>Logout</Label>
        </Button>
    </div>

    <!-- Transfer menu -->
    <form class="column" onsubmit={transferSubmit}>
        <h2>Transfer currency to another user</h2>
        <div>
            <Textfield
                type="username"
                bind:value={transferUserInput}
                label="Username"
                input$autocomplete="username"
                onkeyup={validateTransferUser}
            ></Textfield>
        </div>

        <div>
            <Textfield
                bind:value={transferAmountInput}
                label="Amount"
                input$autocomplete="off"
                onkeyup={validateTransferAmount}
            ></Textfield>
        </div>

        <div class="row">
            <Button variant="raised" type="submit">
                <Label>Transfer</Label>
            </Button>
        </div>

        {#if transferError.length != 0}
            <p class="error">{transferError}</p>
        {/if}
    </form>
{/if}

<style>
    .column {
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
