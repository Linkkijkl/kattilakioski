<script lang="ts">
    import Button, { Label } from "@smui/button";
    import Textfield from "@smui/textfield";

    let username = $state("");
    let password = $state("");
</script>

<div class="login-form">
    <div class="login-field">
        <Textfield
            type="username"
            bind:value={username}
            label="Username"
            input$autocomplete="username"
        >
        </Textfield>
    </div>

    <div class="login-field">
        <Textfield
            type="password"
            bind:value={password}
            label="Password"
            input$autocomplete="new-password"
        >
        </Textfield>
    </div>

    <div class="login-field">
        <Button variant="raised">
            <Label>Login</Label>
        </Button>
    </div>
</div>

<style>
    .login-form {
        display: flex;
        flex-direction: column;
        justify-content: center;
        width: 100%;
        align-items: center;
    }
</style>