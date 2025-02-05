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
	import IconButton, { Icon } from "@smui/icon-button";
	import LayoutGrid, { Cell } from "@smui/layout-grid";

	let {
		title = "no title",
		description = "no description",
		image = "https://placehold.co/320x180?text=16x9",
		price = NaN,
		stock = NaN,
		preview = false,
	} = $props();

	const buy = () => {
		console.log("Not implemented!");
	}
</script>

<div class="card-display">
	<div class="card-container">
		<Card>
			<PrimaryAction onclick={buy}>
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
