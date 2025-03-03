<script lang="ts">
	import Card, {
		Content,
		PrimaryAction,
		Media,
		MediaContent,
		Actions,
		ActionButtons,
		ActionIcons,
	} from "@smui/card";
	import Button, { Label } from "@smui/button";
    import api from "../api.svelte";
    import { createEventDispatcher } from "svelte";
	import { mainDialog } from "../globals.svelte";

	const dispatch = createEventDispatcher();

	let {
		title = "no title",
		description = "no description",
		image = "https://placehold.co/320x180?text=16x9",
		price = "",
		stock = NaN,
		preview = false,
		id = NaN,
	} = $props();

	const buy = async () => {
		try {
			mainDialog.title = "Buy Item";
			mainDialog.content = `Are you sure you want to buy ${title} for ${price}€?`;
			mainDialog.confirmText = "yes";
			mainDialog.cancelText = "no";
			mainDialog.onCancel = () => {};
			mainDialog.onConfirm = () => {
				console.log("wtf");
				api.buyItem({amount: 1, item_id: id});
				console.log("otf");
				dispatch("buyEvent");
			};
			mainDialog.isOpen = true;
		} catch (err: any) {
			alert(err.toString());
		}
		await api.update();
	};

	const view = async () => {
		console.log("Unimplemented");
	};
</script>

<div class="card-display">
	<div class="card-container">
		<Card>
			<PrimaryAction onclick={view}>
				<Media
					class="card-media-16x9"
					aspectRatio="16x9"
					style="background-image: url({image})"
				/>
				<Content class="mdc-typography--body2">
					<div class="content">
						<div>
							<h2 class="mdc-typography--headline6">
								{title}
							</h2>
							{description}
						</div>
						<div>
							{price}€
						</div>
					</div>
				</Content>
			</PrimaryAction>
			{#if !preview}
				<Actions>
					<ActionButtons>
						<Button onclick={buy}>
							<Label>Buy</Label>
						</Button>
					</ActionButtons>
				</Actions>
			{/if}
		</Card>
	</div>
</div>

<style>
	h2 {
		margin: 0;
	}
	.content {
		display: flex;
		justify-content: space-between;
	}
</style>
