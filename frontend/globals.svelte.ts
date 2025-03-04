export const mainDialog = $state({
    isOpen: false,
    title: '',
    content: '',
    confirmText: 'Confirm',
    cancelText: 'Cancel',
    onConfirm: () => { },
    onCancel: () => { } 
});

export const mainBanner = $state({
    isOpen: false,
    message: ''
});
