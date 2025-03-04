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
    import { mdiClose } from "@mdi/js";
    import IconButton, { Icon } from "@smui/icon-button";

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
    aria-labelledby="item-title"
    aria-describedby="item-content"
    surface$style="width: 850px; max-width: calc(100vw - 32px);"
>
    <Header>
        <div class="header-content">
            <Title id="item-title">{title}</Title>
            <IconButton action="close" onclick={() => { isOpen = false; }}>
                <Icon tag="svg" viewBox="0 0 24 24" style="text-align: right;">
                    <path fill="currentColor" d={mdiClose} />
                </Icon>
            </IconButton>
        </div>
    </Header>
    <Content id="item-content" >
        <div class="main-content">
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
                <p class="description">
                    {#if description.length == 0}No description available.{/if}
                    {description}
                </p>
                <div class="infobar">
                    <div class="seller">By: user_id {seller_name}</div>
                    <div class="stock">Stock: {stock}</div>    
                </div>
            </div>
        </div>
    </Content>
    <Actions>
        <div class="actions">
            <b class="price">{price}€</b>
            <!-- TODO: Add Item amount field here -->
            <Button onclick={buy}>Buy</Button>
        </div>
    </Actions>
    
</Dialog>

<style>
    div {
        margin: 0;
        padding: 0;
    }

    .header-content {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding-right: 10px;
    }

    .main-content {
        display: flex;
        flex-direction: column;
        column-gap: 32px;
    }

    @media (min-width: 700px) {
        .main-content {
            flex-direction: row;
        }
    }

    .images {
        width: 100%;
        display: flex;
        flex-direction: column;
    }

    .big-image {
        max-width: 100%;
    }

    .thumbnails {
        display: flex;
        flex-wrap: wrap;
    }

    .thumbnail {
        max-width: 25%;
    }

    .info {
        margin-top: 10px;
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .infobar {
        display: flex;
        justify-content: space-between;
    }

    .title {
        font-size: 2rem;
        font-weight: bold;
        color: #333;
    }

    .description {
        font-size: 1.2rem;
        line-height: 1.5;
    }

    .actions {
        display: flex;
        flex-direction: right;
        align-items: center;
    }

    .price {
        margin: 0;
    }
</style>