<script lang='ts'>
    import ItemCard from "./ItemCard.svelte";
    import { getItems } from "../api";
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
        />
    {/each}
{:catch error}
    <p>Something went wrong: {error.message}</p>
{/await}
