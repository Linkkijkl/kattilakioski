<script lang="ts">
    import Banner, { Label } from "@smui/banner";
    import Button from "@smui/button";
    import { mainBanner } from "../globals.svelte";

    const onClose = () => {
        mainBanner.isOpen = false;
    };
</script>

<Banner
    bind:open={mainBanner.isOpen}
    mobileStacked
    content$style="max-width: max-content;"
>
    {#snippet label()}
        <Label>{mainBanner.message}</Label>
    {/snippet}
    {#snippet actions()}
        <Button onclick={onClose}>Close</Button>
    {/snippet}
</Banner>
