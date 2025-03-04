<script lang="ts">
    import Dialog, { Title, Content, Actions } from "@smui/dialog";
    import Button, { Label } from "@smui/button";
    import { mainDialog } from "../globals.svelte";

    const onAction = () => {
        mainDialog.isOpen = false; 
    };

</script>

<Dialog
    bind:open={mainDialog.isOpen}
    aria-labelledby="simple-title"
    aria-describedby="simple-content"
>
    <Title id="simple-title">{mainDialog.title}</Title>
    <Content id="simple-content">{mainDialog.content}</Content>
    <Actions>
        <Button onclick={() => {
            mainDialog.onCancel();
            onAction();
        }}>
            <Label>{mainDialog.cancelText}</Label>
        </Button>
        <Button onclick={() => {
            mainDialog.onConfirm();
            onAction();
        }}>
            <Label>{mainDialog.confirmText}</Label>
        </Button>
    </Actions>
</Dialog>
