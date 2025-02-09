<script lang='ts'>
    import ItemCard from "./ItemCard.svelte";
    import { getItems } from "../api.svelte";
    let { searchTerm="" } = $props();
    let itemsPromise = getItems({search_term: searchTerm, limit: null, offset: null});
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
        />
    {/each}
{:catch error}
    <p>Something went wrong: {error.message}</p>
{/await}
