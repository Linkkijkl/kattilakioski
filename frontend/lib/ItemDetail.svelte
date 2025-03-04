<script lang="ts">
    import api from "../api.svelte";
    import type { Attachment } from "../api.svelte";
    import Dialog, {
        Header,
        Title,
        Content,
        Actions,
        InitialFocus,
    } from "@smui/dialog";
    import Button, { Label } from "@smui/button";
    import List, { Item, Graphic, Text } from "@smui/list";

	let {
        isOpen = false,
		title = "no title",
		description = "no description",
		attachments = [],
		price = "",
		stock = NaN,
        seller_name = "no seller",
		onBuy = () => {},
	} = $props();

    let selectedAttachmentIndex = $state(0);

    const buy = () => {
        console.log("Unimplemented");
    };

</script>

<Dialog
    bind:open = {isOpen}
    fullscreen
    aria-labelledby="over-fullscreen-title"
    aria-describedby="over-fullscreen-content"
>
    {#if attachments.length > 0}
        <div class="images">
            <img src="{attachments[selectedAttachmentIndex].file_path}" alt="Item being sold" class="big-image"/>
            <div class="thumbnails">
                {#each attachments as attachment, index}
                    <img src={attachment.thumbnail_path} alt="Thumbnail {index+1}" class="thumbnail"/>
                {/each}
            </div>
        </div>
    {/if}
    <div class="info">
        <div class="title">{title}</div>
        <div class="seller">Seller: {seller_name}</div>
        <div class="price">{price}€</div>
        <div class="description">{description}</div>
        <div class="stock">Stock: {stock}</div>
        <div class="actions">
            <Button onclick={buy}>Buy</Button>
        </div>
    </div>
</Dialog>

<style>
    .item-detail {
        display: flex;
    }

    .images {
        display: flex;
    }

    .big-image {
       
    }

    .thumbnails {
        display: flex;
    }

    .thumbnail {

    }

    .info {
        display: flex;
    }

    .title {
        font-size: 1.5rem;
        font-weight: bold;
    }

    .seller {

    }

    .price {
    
    }

    .description {
    
    }

    .stock {

    }

    .actions {

    }
</style>