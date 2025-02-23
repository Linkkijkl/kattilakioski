<script lang="ts">
    import Button, { Label } from "@smui/button";
    import Textfield from "@smui/textfield";

    import { userInfo, updateAPI, adminGive, adminPromote, getUserInfo } from "../api.svelte";
    import { onMount } from "svelte";

    let giveUserInput = $state("");
    let giveAmountInput = $state("");
    let giveError = $state("");
    let promoteUserInput = $state("");
    let promoteError = $state("");

    const giveSubmit = async (event: Event) => {
        event.preventDefault();
        const amountCents = Math.floor(parseFloat(giveAmountInput) * 100);
        try {
            const user = await getUserInfo(giveUserInput);
            await adminGive({user_id: user.id, amount_cents: amountCents});
            giveError = "Success!";
            updateAPI();
        } catch (err: any) {
            giveError = err.toString();
        }
    };

    const promoteSubmit = async (event: Event) => {
        event.preventDefault();
        const amountCents = Math.floor(parseFloat(giveAmountInput) * 100);
        try {
            const user = await getUserInfo(giveUserInput);
            await adminPromote(user.id);
            promoteError = "Success!";
            updateAPI();
        } catch (err: any) {
            promoteError = err.toString();
        }
    };

    onMount(updateAPI);
</script>

<form class="column" onsubmit={giveSubmit}>
    <h1>
        Give currency to user
    </h1>
    <div>
        <Textfield
            type="username"
            bind:value={giveUserInput}
            label="Username"
            placeholder={userInfo.username}
        ></Textfield>
    </div>

    <div>
        <Textfield
            type="amount"
            bind:value={giveAmountInput}
            label="Amount"
            placeholder="0.00"
        ></Textfield>
    </div>

    <div>
        <Button variant="raised" type="submit">
            <Label>Give</Label>
        </Button>
    </div>

    {#if giveError.length != 0}
        <p class="error">{giveError}</p>
    {/if}
</form>

<form class="column" onsubmit={promoteSubmit}>
    <h1>
        Promote user to admin status
    </h1>
    <div>
        <Textfield
            type="username"
            bind:value={promoteUserInput}
            label="Username"
            placeholder={userInfo.username}
        ></Textfield>
    </div>

    <div>
        <Button variant="raised" type="submit">
            <Label>Promote</Label>
        </Button>
    </div>

    {#if promoteError.length != 0}
        <p class="error">{promoteError}</p>
    {/if}
</form>

<style>
    .column {
        display: flex;
        flex-direction: column;
        justify-content: center;
        width: 100%;
        align-items: center;
    }
    /*.row {
        display: flex;
        flex-direction: row;
    }*/
    .error {
        color: red;
    }
</style>
