export type UserQuery = {
    username: string,
    password: string
};

export type User = {
    id: number,
    username: string,
    balance_cents: number,
    created_at: Date,
    is_admin: boolean,
};

export type ItemQuery = {
    search_term: string | null,
    offset: number | null,
    limit: number | null,
    get_items_without_stock: boolean | null,
};

export type Attachment = {
    id: number,
    file_path: string,
    thumbnail_path: string,
    item_id: number | null,
    uploader_id: number,
    uploaded_at: Date
};

export type ItemResult = {
    id: number,
    title: string,
    description: string,
    price_cents: number,
    amount: number,
    seller_id: number,
    created_at: Date,
    attachments: Attachment[]
};

export type NewItemQuery = {
    title: string,
    description: string,
    amount: number,
    price: string,
    attachments: number[],
};

export type BuyQuery = {
    item_id: number,
    amount: number,
};

export type AdminGiveQuery = {
    user_id: number | null,
    amount_cents: number,
};

export type ValidateQuery = {
    value: string,
}
