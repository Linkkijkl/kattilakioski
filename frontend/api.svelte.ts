import type { User, UserQuery, ItemResult, ItemQuery, NewItemQuery, BuyQuery, Attachment, AdminGiveQuery, ValidateQuery } from './types';

const apiUrl = '/api';
const headers = { 'Content-Type': 'application/json' };

export const userInfo = $state({ isLoggedIn: false, username: '', balance: "0", isAdmin: false });

const api = (() => {
    /**
     * Updates API runes with fresh information from the server
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
     * @param {string} type - The type of validation to perform. Must be one of 'password', 'currency', or 'username'.
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
        login, logout, newUser, getUserInfo, update, getItems, newItem, newAttachment, buyItem, adminGive, adminPromote, validate
    };
})();

export default api;
