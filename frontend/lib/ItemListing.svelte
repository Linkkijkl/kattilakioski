<script lang='ts'>
    import ItemCard from "./ItemCard.svelte";
    import api from "../api.svelte";
    import type { ItemResult } from "../api.svelte";
    let { searchTerm="" } = $props();
    let itemsPromise: Promise<ItemResult[]> = $state(Promise.resolve([]));
    const update = () => itemsPromise = api.getItems({search_term: searchTerm, limit: null, offset: null, get_items_without_stock: false});
    update();
</script>

{#await itemsPromise}
    <p>loading...</p>
{:then items}
    <div class="items">
        {#each items as item}
            <div class="item">
                <ItemCard 
                    title={item.title}
                    description={item.description}
                    price={(item.price_cents / 100.0).toString()}
                    stock={item.amount}
                    image={item.attachments[0].thumbnail_path}
                    id={item.id}
                    on:buyEvent={update}
                />
            </div>
        {/each}
    </div>
{:catch error}
    <p>Something went wrong: {error.message}</p>
{/await}

<style>
    :root {
        --xs-columns: 1;
        --sm-columns: 2;
        --md-columns: 3;
        --lg-columns: 4;
        --xl-columns: 5;
        --xxl-columns: 6;
        --gap: 5px;
    }

    @media (max-width: 576px) {
        :root {
            --columns: var(--xs-columns);
        }
    }

    @media (min-width: 576px) {
        :root {
            --columns: var(--sm-columns);
        }
    }

    @media (min-width: 768px) {
        :root {
            --columns: var(--md-columns);
        }
    }

    @media (min-width: 992px) {
        :root {
            --columns: var(--lg-columns);
        }
    }

    @media (min-width: 1200px) {
        :root {
            --columns: var(--xl-columns);
        }
    }

    @media (min-width: 1400px) {
        :root {
            --columns: var(--xxl-columns);
        }
    }

    .items {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        gap: var(--gap);
        padding-top: var(--gap);
    }

    .item {
        flex-shrink: 1;
        flex-basis: calc(100% / var(--columns) - (var(--columns) - 1 ) * (var(--gap) / var(--columns)));
    }
</style>