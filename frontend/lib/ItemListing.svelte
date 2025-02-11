<script lang='ts'>
    import ItemCard from "./ItemCard.svelte";
    import { getItems } from "../api.svelte";
    import type { ItemResult } from "../api.svelte";
    let { searchTerm="" } = $props();
    let itemsPromise: Promise<ItemResult[]> = $state();
    const update = () => itemsPromise = getItems({search_term: searchTerm, limit: null, offset: null});
    update();
</script>

{#await itemsPromise}
    <p>loading...</p>
{:then items}
    {#each items as item}
        <ItemCard 
            title={item.title}
            description={item.description}
            price={item.price_cents / 100.0}
            stock={item.amount}
            image={item.attachments[0].thumbnail_path}
            id={item.id}
            on:buyEvent={update}
        />
    {/each}
{:catch error}
    <p>Something went wrong: {error.message}</p>
{/await}
