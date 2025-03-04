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
    aria-labelledby="over-fullscreen-title"
    aria-describedby="over-fullscreen-content"
    surface$style="width: 850px; max-width: calc(100vw - 32px);"
>
    <Header>
        <Title id="over-fullscreen-title">{title}</Title>
        <!-- <IconButton action="close" class="material-icons">close</IconButton> -->
    </Header>
    <Content id="over-fullscreen-content" >
        <div class="main-container">
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
                <div class="description">
                    {#if description.length == 0}No description available.{/if}
                    {description}
                </div>
                <div class="infobar">
                    <div class="seller">By: user_id {seller_name}</div>
                    <div class="stock">Stock: {stock}</div>    
                </div>
            </div>
        </div>
    </Content>
    <Actions>
        <Button onclick={buy}>Buy</Button>
    </Actions>
    
</Dialog>

<style>
    div {
        margin: 0;
        padding: 0;
    }

    .main-container {
        display: flex;
        flex-direction: column;
        column-gap: 32px;
    }

    @media (min-width: 700px) {
        .main-container {
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
        margin-top: 30px;
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
        margin-bottom: 10px;
    }

    .description {
        font-size: 1.2rem;
        line-height: 1.5;
    }
</style>