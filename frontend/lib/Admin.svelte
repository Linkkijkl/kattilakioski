<script lang="ts">
    import Button, { Label } from "@smui/button";
    import Textfield from "@smui/textfield";

    import { userInfo, updateAPI, adminGive, adminPromote } from "../api.svelte";
    import { onMount } from "svelte";

    onMount(updateAPI);
</script>

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
