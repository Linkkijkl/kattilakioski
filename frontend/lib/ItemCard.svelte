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

	let clicked = $state(0);
	let {
		title = "no title",
		description = "no description",
		image = "https://placehold.co/320x180?text=16x9",
		price = NaN,
		stock = NaN,
	} = $props();
</script>

<div class="card-display">
	<div class="card-container">
		<Card>
			<PrimaryAction onclick={() => clicked++}>
				<Media class="card-media-16x9" aspectRatio="16x9" background-image={image}/>
				<Content class="mdc-typography--body2">
					<LayoutGrid>
						<Cell span={6}>
							<div class="card-cell">
								<h2 class="mdc-typography--headline6" style="margin: 0;">
									{title}
								</h2>
							</div>
							{description}
						</Cell>
						<Cell span={1}>
							<div class="card-cell">
								{price}€
							</div>
						</Cell>
					</LayoutGrid>
				</Content>
			</PrimaryAction>
			<Actions>
				<ActionButtons>
					<Button onclick={() => clicked++}>
						<Label>Buy</Label>
					</Button>
				</ActionButtons>
			</Actions>
		</Card>
	</div>
</div>

<style>
	* :global(.card-media-16x9) {
		background-image: url(https://placehold.co/320x180?text=16x9);
	}
	.card-cell {
		display: flex;
	}
</style>
