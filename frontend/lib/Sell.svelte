<script lang="ts">
    import Button, { Label } from "@smui/button";
    import Textfield from "@smui/textfield";
    import ItemCard from "./ItemCard.svelte";
    import api from "../api.svelte";
    import CircularProgress from '@smui/circular-progress';

    let title = $state("");
    let description = $state("");
    let amount = $state("1");
    let price = $state("0.0");
    let files: FileList | any = $state();
    let imageDataUrl = $state("");
    let error: string = $state("");
    let progress: boolean = $state(false);

    const debounceTimeout = 500;

    const sell = async (event: Event) => {
        event.preventDefault();
        progress = true;
        let attachments = [];
        for (const file of files) {
            try {
                let response = await api.newAttachment(file);
                attachments.push(response.id);
                await api.newItem({title, amount: parseInt(amount), attachments, description, price});
                error = "Success!";
                await api.update();
            } catch (err: any) {
                error = err.toString();
            }
        }
        progress = false;
    };

    $effect(() => {
        if (files != null && files.length > 0) {
            const reader = new FileReader();
            reader.addEventListener("load", () => {
                const result = reader.result;
                if (typeof result != "string") {
                    return;
                };
                imageDataUrl = result;
            });
            reader.readAsDataURL(files[0]);
        }
    });

    let priceTimer: number;
    const validatePrice = () => {
        clearTimeout(priceTimer);
        priceTimer = setTimeout(async () => {
            try {
                await api.validate('currency', price);
                error = "";
            } catch (err: any) {
                error = err.toString();
            }
        }, debounceTimeout);
    };
</script>

<form class="upload-form" onsubmit={sell}>
    <Textfield
        style="width: 100%;"
        helperLine$style="width: 100%"
        bind:value={title}
        label="Title"
    ></Textfield>

    <Textfield
        bind:value={description}
        label="Description"
        textarea
        style="width: 100%;"
        helperLine$style="width: 100%;"
    ></Textfield>

    <div class="row">
        <Textfield label="Price" bind:value={price} onkeyup={validatePrice}></Textfield>
        <Textfield label="Amount" bind:value={amount}></Textfield>
    </div>

    <input
        accept="image/png, image/jpeg, image/webp"
        bind:files
        type="file"
        id="attachments"
        multiple
    />

    {#each files as file}
        <p>{file.name}</p>
    {/each}

    <Button variant="raised" type="submit">
        <Label>Sell</Label>
    </Button>

    {#if progress}
        <CircularProgress style="height: 32px; width: 32px;" indeterminate />
    {/if}

    {#if error.length != 0}
        <p class="error">{error}</p>
    {/if}
</form>

<div class="upload-form">
    <Label>Preview</Label>
</div>

<ItemCard
    {title}
    {description}
    stock={parseInt(amount)}
    {price}
    image={imageDataUrl}
    preview={true}
/>

<style>
    .upload-form {
        display: flex;
        flex-direction: column;
        justify-content: center;
        width: 100%;
        align-items: center;
        margin-bottom: 2em;
    }
    .row {
        display: flex;
        flex-direction: row;
        justify-content: center;
        width: 100%;
    }
</style>
