const apiUrl = '/api';
const headers = { 'Content-Type': 'application/json' };

type UserQuery = {
    username: string,
    password: string
};

export const userInfo = $state({ isLoggedIn: false, username: '', balance: "0", isAdmin: false });

/**
 * Updates API runes with fresh information from the server
 */
const updateAPI = async (): Promise<void> => {
    try {
        const user: User = await getUserInfo();
        console.log(user);
        userInfo.isLoggedIn = true;
        userInfo.username = user.username;
        userInfo.balance = (user.balance_cents / 100.0).toString();
        userInfo.isAdmin = user.is_admin;
    } catch (err) {
        console.log(err.toString());
        userInfo.isLoggedIn = false;
    }
};

/**
 * Logs user in with given information
 * @param {UserQuery} query Login information
 */
const login = async (query: UserQuery): Promise<void> => {
    const response = await fetch(`${apiUrl}/user/login`, {
        body: JSON.stringify(query),
        method: 'POST',
        headers,
    });
    if (response.status != 200) {
        throw new Error(await response.text());
    }
    userInfo.username = query.username;
    userInfo.isLoggedIn = true;
    userInfo.isAdmin = (await response.json()).is_admin;
};

/**
 * Logs user out
 */
const logout = async (): Promise<void> => {
    const response = await fetch(`${apiUrl}/user/logout`);
    if (response.status != 200) {
        throw new Error(await response.text());
    }
    userInfo.username = null;
    userInfo.isLoggedIn = false;
};

/**
 * Creates a new user
 * @param {UserQuery} query Login information
 */
const newUser = async (query: UserQuery): Promise<void> => {
    const response = await fetch(`${apiUrl}/user/new`, {
        body: JSON.stringify(query),
        method: 'POST',
        headers,
    });
    if (response.status != 200) {
        throw new Error(await response.text());
    }
};

type User = {
    id: number,
    username: string,
    balance_cents: number,
    created_at: Date,
    is_admin: boolean,
};

/**
 * Retrieves user info
 * @param {Number | string | null} user
 *      Username or user id to get info for.
 *      Set null to get info for currently logged in user.
 * @returns {User} User information
 */
const getUserInfo = async (user: number | string | null = null): Promise<User> => {
    // Return info for self if no user was provided
    if (user == null) {
        const response = await fetch(`${apiUrl}/user`, {
            method: 'POST',
            headers,
        });
        if (response.status != 200) {
            throw new Error(await response.text());
        }
        return await response.json();
    }

    // Construct query from provided username or user id
    let body: any;
    if (isNaN(+user)) {
        body = { 'Username': user };
    } else {
        body = { 'UserId': user };
    }

    // Get info for provided user
    const response = await fetch(`${apiUrl}/user`, {
        body: JSON.stringify(body),
        method: 'POST',
        headers,
    });

    if (response.status != 200) {
        throw new Error(await response.text());
    }

    return await response.json();
};

type ItemQuery = {
    search_term: string | null,
    offset: number | null,
    limit: number | null,
    get_items_without_stock: boolean | null,
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
        query = { limit: null, offset: null, search_term: null, get_items_without_stock: false };
    }
    const response = await fetch(`${apiUrl}/item/list`, {
        body: JSON.stringify(query),
        method: 'POST',
        headers,
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
    const response = await fetch(`${apiUrl}/item/new`, {
        body: JSON.stringify(query),
        method: 'POST',
        headers,
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
    formData.append('file', file);
    const response = await fetch(`${apiUrl}/attachment/upload`, {
        body: formData,
        method: 'POST',
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
const buyItem = async (query: BuyQuery): Promise<void> => {
    const response = await fetch(`${apiUrl}/item/buy`, {
        body: JSON.stringify(query),
        method: 'POST',
        headers,
    });
    if (response.status != 200) {
        throw new Error(await response.text());
    }
};

/**
 * Promotes user to admin status. Tries to promote user logged in if no
 * user is proveded. The endpoint normally requires admin status, but
 * in debug builds does not.
 * @param {Number | null} userId User id to promote
 */
const adminPromote = async (userId: Number | null = null): Promise<void> => {
    const response = await fetch(`${apiUrl}/item/buy`, {
        body: JSON.stringify({
            user_id: userId
        }),
        method: 'POST',
        headers,
    });
    if (response.status != 200) {
        throw new Error(await response.text());
    }
};

type AdminGiveQuery = {
    user_id: Number | null,
    admount_cents: Number,
};

/**
 * Gives user currency. Adds currency to currently logged in user if no
 * user is provided. The endpoint normally requires admin status, but
 * in debug builds does not.
 */
const adminGive = async (query: AdminGiveQuery): Promise<void> => {
    const response = await fetch(`${apiUrl}/admin/give`, {
        body: JSON.stringify(query),
        method: 'POST',
        headers,
    });
    if (response.status != 200) {
        throw new Error(await response.text());
    }
};

export type {
    BuyQuery, ItemQuery, NewItemQuery, UserQuery, ItemResult
};
export {
    login, logout, newUser, getUserInfo, updateAPI, getItems, newItem, newAttachment, buyItem, adminGive, adminPromote
};
