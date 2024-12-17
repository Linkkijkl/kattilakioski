# Key-Value Database Schema for Kattilakioski


```

# All ID's are integers


user-id:<username> = User id : unsigned integer

user:<user-id>:salt = Salt appended in front of password before hashing : string
user:<user-id>:password-hash = Hashed user password : string
user:<user-id>:session-id = Session id for user : string
user:<user-id>:balance-cents = User balance in cents : positive integer
user:<user-id>:attachments = List of attachments associated with this user : List<attachment-id>
user:<user-id>:creation-timestamp = Unix timestamp of creation : unsigned integer

admins = List of all admins by user id : List<user-id>

item:<item-id>:name = Name of item : string
item:<item-id>:price-cents = Cost of item in cents : positive integer
item:<item-id>:stock = Current amount of this item in stock : positive integer
item:<item-id>:attachments = List of attachments associated with this item : List<attachment-id>

attachment:<attachment-id>:thumbnail-path = Thumbnail path for this attachment
attachment:<attachment-id>:path = Path for this attachment : string
attachment:<attachment-id>:upload-timestamp = Unix timestamp of upload : unsigned integer

```


# SQL Database Schema for Kattilakioski

diesel::table! {
    users (id) {
        id -> Int4,
        username -> VarChar,
        password_salt -> VarChar,
        password_hash -> VarChar,
        balance_cents -> Integer,
        is_admin -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    items (id) {
        id -> Int4,
        title -> VarChar,
        description -> VarChar,
        price_cents -> Integer,
        amount -> Int4,
        seller_user_id -> Int4,
    }
}

diesel::table! {
    transactions (id) {
        id -> Int4,
        from_user_id -> Int4,
        to_user_id -> Int4,
        price_cents -> Integer,
    }
}

diesel::joinable!(transactions -> users(to_user_id));

diesel::joinable!(transactions -> users(from_user_id));

diesel::table! {
    attachments (id) {
        id -> Int4,
        user_id -> Int4,
    }
}

diesel::joinable!(attachments -> users(user_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    attachments,
);


# API Endpoints

GET /api/hello -> 200 { Hello World }

GET /