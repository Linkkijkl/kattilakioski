const apiUrl = '/api';
const headers = { 'Content-Type': 'application/json' };

export const userInfo = $state({ isLoggedIn: false, username: '', balance: "0", isAdmin: false });

// Types defined here should represent types defined in the backend

/**
 * Represents the query parameters for user authentication and registration.
 */
export type UserQuery = {
    username: string,
    password: string
};

/**
 * Represents a user and their information.
 */
export type User = {
    id: number,
    username: string,
    balance_cents: number,
    created_at: Date,
    is_admin: boolean,
};

/**
 * Represents the query parameters for searching items.
 */
export type ItemQuery = {
    search_term: string | null,
    offset: number | null,
    limit: number | null,
    get_items_without_stock: boolean | null,
};

/**
 * Represents an attachment object.
 */
export type Attachment = {
    id: number,
    file_path: string,
    thumbnail_path: string,
    item_id: number | null,
    uploader_id: number,
    uploaded_at: Date
};

/**
 * Represents an item.
 */
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

/**
 * Represents the parameters for creating a new item.
 */
export type NewItemQuery = {
    title: string,
    description: string,
    amount: number,
    price: string,
    attachments: number[],
};

/**
 * Represents the query parameters for buying an item.
 */
export type BuyQuery = {
    item_id: number,
    amount: number,
};

/**
 * Represents the query parameters for giving balance to a user.
 */
export type AdminGiveQuery = {
    user_id: number | null,
    amount_cents: number,
};

/**
 * Represents the query parameters for validating a form value.
 */
export type ValidateQuery = {
    value: string,
}

const api = (() => {    
    /**
     * Updates `userInfo` with fresh information from the server
     */
    const update = async (): Promise<void> => {
        try {
            const user: User = await getUserInfo();
            userInfo.isLoggedIn = true;
            userInfo.username = user.username;
            userInfo.balance = (user.balance_cents / 100.0).toString();
            userInfo.isAdmin = user.is_admin;
        } catch (err) {
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
        userInfo.username = '';
        userInfo.isLoggedIn = false;
        userInfo.isAdmin = false;
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

    /**
     * Get list of items for sale. List can be filtered, limited and offset by provided ItemQuery.
     * @param {ItemQuery | null} query Optional request filtering, result limiting and offset information.
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
        const response = await fetch(`${apiUrl}/admin/promote`, {
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

    /**
     * Validates a given value against a specific type of field.
     * @param {string} type - The type of validation to perform. Must be one of 'password', 'currency',
     * or 'username'.
     * @param {string} value - The value to validate.
     * @throws Will throw an error if the validation type is invalid.
     */
    const validate = async (type: string, value: string): Promise<void> => {
        if (!['password', 'currency', 'username'].includes(type)) {
            throw new Error('Invalid validation type');
        }
        const body: ValidateQuery = { value };
        const response = await fetch(`${apiUrl}/validate/${type}`, {
            body: JSON.stringify(body),
            method: 'POST',
            headers,
        });
        const response_body = await response.text();
        if (response_body != 'OK') {
            throw new Error(response_body);
        }
    };

    return {
        login, logout, newUser, getUserInfo, update, getItems, newItem, newAttachment, buyItem, adminGive,
        adminPromote, validate
    };
})();

export default api;
