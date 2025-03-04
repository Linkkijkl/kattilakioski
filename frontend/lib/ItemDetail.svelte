<script lang="ts">
    import api from "../api.svelte";
    import type { Attachment } from "../api.svelte";
    import Dialog, {
        Header,
        Title,
        Content,
        Actions,
    } from "@smui/dialog";
    import Button from "@smui/button";
    import { mdiClose } from "@mdi/js";
    import IconButton, { Icon } from "@smui/icon-button";
    import { mainDialog, mainBanner } from "../globals.svelte";

	let {
        isOpen = false,
		title = "no title",
		description = "no description",
		attachments = [],
		price = "",
		stock = NaN,
        seller_name = "no seller",
		onBuy = () => {},
        id = NaN,
	} = $props();

    let selectedAttachmentIndex = $state(0);

	const buy = async () => {
		mainDialog.title = "Buy Item";
		mainDialog.content = `Are you sure you want to buy ${title} for ${price}€?`;
		mainDialog.confirmText = "yes";
		mainDialog.cancelText = "no";
		mainDialog.onCancel = () => {};
		mainDialog.onConfirm = async () => {
			try {
				await api.buyItem({ amount: 1, item_id: id });
				onBuy();
			} catch (err: any) {
				mainBanner.message = err.toString();
				mainBanner.isOpen = true;
			}
		};
		mainDialog.isOpen = true;
		await api.update();
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
                    {#if attachments.length > 1}
                        <div class="thumbnails">
                            {#each attachments as attachment, index}
                                <img src={attachment.thumbnail_path} alt="Thumbnail {index+1}" class="thumbnail"/>
                            {/each}
                        </div>
                    {/if}
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