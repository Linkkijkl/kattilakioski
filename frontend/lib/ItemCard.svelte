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
    import { buyItem, updateAPI } from "../api.svelte";
    import { createEventDispatcher } from "svelte";

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
			await buyItem({amount: 1, item_id: id});
			dispatch("buyEvent");
		} catch (err: any) {
			alert(err.toString());
		}
		await updateAPI();
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
