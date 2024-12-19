type UserQuery = {
    username: string,
    password: string
};

let username: string | null = null;

/**
 * Logs user in with given information
 * @param {UserQuery} query Login information
 */
const login = async (query: UserQuery): Promise<void> => {
    const response = await fetch("/api/user/login", {
        body: JSON.stringify(query)
    });
    if (response.status != 200) {
        throw new Error(await response.text());
    }
    username = query.username;
};

/**
 * Logs user out
 */
const logout = async (): Promise<void> => {
    const response = await fetch("/api/user/logout");
    if (response.status != 200) {
        throw new Error(await response.text());
    }
    username = null;
};

/**
 * Creates a new user
 * @param {UserQuery} query Login information
 */
const newUser = async (query: UserQuery): Promise<void> => {
    const response = await fetch("/api/user/new", {
        body: JSON.stringify(query)
    });
    if (response.status != 200) {
        throw new Error(await response.text());
    }
};

type User = {
    id: number,
    username: string,
    balance_cents: number,
    created_at: Date
};

/**
 * Retrieves user info
 * @returns {User} User information
 */
const userInfo = async (): Promise<User> => {
    const response = await fetch("/api/user/info");
    if (response.status != 200) {
        throw new Error(await response.text());
    }
    return await response.json();
};

/**
 * Returns username of user logged in. Faster than `userInfo()`.
 * @returns {string | null} Username of user logged in
 */
const getUsername = (): string | null => username;

/**
 * Returns true if user is logged in. Faster than `userInfo()`.
 * @returns {boolean} Is user logged in
 */
const isLoggedIn = (): boolean => username != null;

type ItemQuery = {
    search_term: string | null,
    offset: number | null,
    limit: number | null
};

type Attachment = {
    id: number,
    file_path: string,
    thumbnail_path: string,
    item_id: number | null,
    uploader_id: number,
    uploaded_at: Date
};

type ItemResult = {
    id: number,
    title: string,
    description: string,
    price_cents: number,
    amount: number,
    seller_id: number,
    created_at: Date,
    attachments: Attachment[]
};

/**
 * Get list of items for sale. List can be filtered, limited and offset by provided ItemQuery.
 * @param {ItemQuery | null} query Optional equest filtering, result limiting and offset information.
 * @returns {ItemResult[]} List of items matching query
 */
const getItems = async (query: ItemQuery | null = null): Promise<ItemResult[]> => {
    if (query == null) {
        query = { limit: null, offset: null, search_term: null };
    }
    const response = await fetch("/api/item/list", {
        body: JSON.stringify(query)
    });
    if (response.status != 200) {
        throw new Error(await response.text());
    }
    return await response.json();
};

type AttachmentId = number;

type NewItemQuery = {
    title: String,
    description: String,
    amount: number,
    price: string,
    attachments: AttachmentId[]
};

/**
 * Sells an item with provided information.
 * @param {NewItemQuery} query New item information
 * @returns {ItemResult} Created attachment information
 */
const newItem = async (query: NewItemQuery): Promise<ItemResult> => {
    const response = await fetch("/api/item/new", {
        body: JSON.stringify(query)
    });
    if (response.status != 200) {
        throw new Error(await response.text());
    }
    return await response.json();
};

/**
 * Uploads a new attachment which can be used in `newItem()`
 * @param {File} file Attachment file to be uploaded
 * @returns {Attachment} Created attachment information
 */
const newAttachment = async (file: File): Promise<Attachment> => {
    const formData = new FormData();
    formData.append("file", file);
    const response = await fetch("/api/attachment/upload", {
        body: formData
    });
    if (response.status != 200) {
        throw new Error(await response.text());
    }
    return await response.json();
};

type BuyQuery = {
    item_id: number,
    amount: number,
};

/**
 * Buys an item
 * @param {BuyQuery} query Item id and amount to be bought
 */
const buy = async (query: BuyQuery): Promise<void> => {
    const response = await fetch("/api/item/buy", {
        body: JSON.stringify(query)
    });
    if (response.status != 200) {
        throw new Error(await response.text());
    }
};

export { BuyQuery, ItemQuery, NewItemQuery, UserQuery,
    login, logout, newUser, userInfo, getUsername, isLoggedIn, getItems, newItem, newAttachment, buy };
