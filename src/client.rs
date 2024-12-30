use chrono::{ Duration, Utc };
use jsonwebtoken::{ encode, Algorithm, EncodingKey, Header };
use rand::Rng;
use sec1::{ pkcs8::LineEnding, DecodeEcPrivateKey };
use serde::Serialize;

use crate::{
    accounts::Accounts,
    contract_expiry_type::ContractExpiryType,
    expiring_contract_status::ExpiringContractStatus,
    granularity::Granularity,
    market_trades::MarketTrades,
    price_books::PriceBooks,
    product::Product,
    product_book::ProductBook,
    product_candles::ProductCandles,
    product_type::ProductType,
    products::Products,
    server_time::ServerTime,
};

pub struct Client<'a> {
    pub name: &'a str,
    client: reqwest::Client,
}

impl<'a> Client<'_> {
    pub fn new(name: &'a str) -> Client<'a> {
        Client {
            name,
            client: reqwest::Client::new(),
        }
    }

    async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        let result = self.client.get(url).send().await;
        if let Ok(response) = &result {
            if !response.status().is_success() {
                println!("Failed: {}", response.status());
            }
        } else if let Err(e) = &result {
            println!("Failed to get response: {}", e);
        }
        result
    }

    async fn get_auth(&self, url: &str, jwt: &str) -> Result<reqwest::Response, reqwest::Error> {
        let result = self.client.get(url).bearer_auth(jwt).send().await;
        if let Ok(response) = &result {
            if !response.status().is_success() {
                println!("Failed: {}", response.status());
            }
        } else if let Err(e) = &result {
            println!("Failed to get response: {}", e);
        }
        result
    }

    pub async fn list_accounts(&self) -> Result<Accounts, reqwest::Error> {
        let url = &format!("{}", PUBLIC_ACCOUNTS_URL);
        let response = self.get_auth(url, &create_jwt("GET", PUBLIC_ACCOUNTS_ENDPOINT)).await?;
        let accounts: Accounts = response.json().await?;
        Ok(accounts)
    }

    pub async fn get_account(&self, account_uuid: &str) -> Result<Accounts, reqwest::Error> {
        let url = &format!("{}/{}", PUBLIC_ACCOUNTS_URL, account_uuid);
        let response = self.get_auth(
            url,
            &create_jwt("GET", &format!("{}/{}", PUBLIC_ACCOUNTS_ENDPOINT, account_uuid))
        ).await?;
        let accounts: Accounts = response.json().await?;
        Ok(accounts)
    }

    pub async fn get_public_market_trades(
        &self,
        product_id: &str,
        limit: u32,
        start: Option<String>,
        end: Option<String>
    ) -> Result<MarketTrades, reqwest::Error> {
        let start = match start {
            Some(start) => &format!("&start={}", start),
            None => "",
        };
        let end = match end {
            Some(end) => &format!("&end={}", end),
            None => "",
        };
        let url = &format!(
            "{}{}/ticker?limit={}{}{}",
            PUBLIC_MARKET_TRADES_URL,
            product_id,
            limit,
            start,
            end
        );
        let response = self.get(url).await?;
        let market_trades: MarketTrades = response.json().await?;
        Ok(market_trades)
    }

    pub async fn get_best_bid_ask(
        &self,
        product_ids: Option<Vec<&str>>
    ) -> Result<PriceBooks, reqwest::Error> {
        let mut query_params = Vec::new();
        if let Some(product_ids) = product_ids {
            for product_id in product_ids {
                query_params.push(format!("product_ids={}", product_id));
            }
        }
        let query_string = if query_params.is_empty() {
            String::new()
        } else {
            format!("?{}", query_params.join("&"))
        };
        let url = &format!("{}{}", BEST_BID_ASK_URL, query_string);
        let response = self.get_auth(url, &create_jwt("GET", BEST_BID_ASK_ENDPOINT)).await?;
        let price_books: PriceBooks = response.json().await?;
        Ok(price_books)
    }
    pub async fn get_public_product_book(
        &self,
        product_id: &str,
        limit: Option<u32>,
        aggregation_price_increment: Option<&str>
    ) -> Result<ProductBook, reqwest::Error> {
        let limit = match limit {
            Some(limit) => &format!("&limit={}", limit),
            None => "",
        };
        let aggregation_price_increment = match aggregation_price_increment {
            Some(aggregation_price_increment) =>
                &format!("&aggregation_price_increment={}", aggregation_price_increment),
            None => "",
        };
        let url = &format!(
            "{}?product_id={}{}{}",
            PUBLIC_PRODUCT_BOOK_URL,
            product_id,
            limit,
            aggregation_price_increment
        );
        let response = self.get(url).await?;
        let product: ProductBook = response.json().await?;
        Ok(product)
    }

    pub async fn get_public_product_candles(
        &self,
        product_id: &str,
        start: &str,
        end: &str,
        granularity: Granularity,
        limit: Option<u32>
    ) -> Result<ProductCandles, reqwest::Error> {
        let limit = match limit {
            Some(limit) => &format!("&limit={}", limit),
            None => "",
        };

        let url = &format!(
            "{}/{}/candles?start={}&end={}&granularity={}{}",
            PUBLIC_PRODUCT_URL,
            product_id,
            start,
            end,
            granularity,
            limit
        );
        let response = self.get(url).await?;
        let product_candles: ProductCandles = response.json().await?;
        Ok(product_candles)
    }

    pub async fn get_public_product(&self, product_id: &str) -> Result<Product, reqwest::Error> {
        let url = &format!("{}/{}", PUBLIC_PRODUCT_URL, product_id);
        let response = self.get(url).await?;
        let product: Product = response.json().await?;
        Ok(product)
    }
    pub async fn list_public_products(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        product_type: Option<ProductType>,
        product_ids: Option<Vec<&str>>,
        contract_expiry_type: Option<ContractExpiryType>,
        expiring_contract_status: Option<ExpiringContractStatus>,
        get_all_products: Option<bool>
    ) -> Result<Products, reqwest::Error> {
        let mut query_params = Vec::new();

        if let Some(limit) = limit {
            query_params.push(format!("limit={}", limit));
        }

        if let Some(offset) = offset {
            query_params.push(format!("offset={}", offset));
        }

        if let Some(product_type) = product_type {
            query_params.push(format!("product_type={}", product_type));
        }

        if let Some(product_ids) = product_ids {
            for product_id in product_ids {
                query_params.push(format!("product_ids={}", product_id));
            }
        }

        if let Some(contract_expiry_type) = contract_expiry_type {
            query_params.push(format!("contract_expiry_type={}", contract_expiry_type));
        }

        if let Some(expiring_contract_status) = expiring_contract_status {
            query_params.push(format!("expiring_contract_status={}", expiring_contract_status));
        }

        if let Some(get_all_products) = get_all_products {
            query_params.push(format!("get_all_products={}", get_all_products));
        }

        let query_string = if query_params.is_empty() {
            String::new()
        } else {
            format!("?{}", query_params.join("&"))
        };

        let url = &format!("{}{}", PUBLIC_PRODUCTS_URL, query_string);
        let response = self.get(url).await?;
        let products: Products = response.json().await?;
        Ok(products)
    }

    pub async fn get_public_server_time(&self) -> Result<ServerTime, reqwest::Error> {
        let response = self.get(PUBLIC_SERVER_TIME).await?;
        let server_time: ServerTime = response.json().await?;
        Ok(server_time)
    }
}

#[derive(Debug, Serialize)]
struct Claims {
    sub: String,
    iss: String,
    nbf: i64,
    exp: i64,
    uri: String,
    kid: String,
    nonce: String,
}

fn create_jwt(request_method: &str, request_path: &str) -> String {
    let key_name = std::env
        ::var("CBAT_KEY_NAME")
        .expect("CBAT_KEY_NAME environment variable not set");
    let key_secret = std::env
        ::var("CBAT_KEY_SECRET")
        .expect("CBAT_KEY_SECRET environment variable not set");
    let uri = format!("{} {}{}", request_method, BASE_URL, request_path);

    let mut rng = rand::thread_rng();
    let nonce: String = (0..16)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();

    let now = Utc::now();
    let claims = Claims {
        sub: key_name.to_owned(),
        iss: "cdp".to_owned(),
        nbf: now.timestamp(),
        exp: (now + Duration::seconds(60)).timestamp(),
        uri,
        kid: key_name.to_owned(),
        nonce,
    };
    let header = Header {
        alg: Algorithm::ES256,
        kid: Some(key_name.to_owned()),
        ..Default::default()
    };
    let key_secret = key_secret.replace("\\n", "\n");
    let pem = from_sec1_pem(&key_secret);
    let key = EncodingKey::from_ec_pem(pem.as_bytes()).expect("Invalid EC key");
    let jwt = encode(&header, &claims, &key).unwrap();
    jwt
}

fn from_sec1_pem(pem: &str) -> String {
    let ec_private_key = sec1::pkcs8::SecretDocument::from_sec1_pem(pem).unwrap();
    let pkcs8_pem = ec_private_key.to_pem("PRIVATE KEY", LineEnding::LF);
    let binding = pkcs8_pem.unwrap();
    let pem: &str = binding.as_ref();
    pem.to_string()
}

const PROTOCOL: &str = "https://";
const BASE_URL: &str = "api.coinbase.com";
const PUBLIC_ACCOUNTS_URL: &str = "https://api.coinbase.com/api/v3/brokerage/accounts";
const PUBLIC_ACCOUNTS_ENDPOINT: &str = "/api/v3/brokerage/accounts";
const PUBLIC_MARKET_TRADES_URL: &str = "https://api.coinbase.com/api/v3/brokerage/market/products/";
const PUBLIC_MARKET_TRADES_ENDPOINT: &str = "/api/v3/brokerage/market/products/ticker";
const PUBLIC_PRODUCT_URL: &str = "https://api.coinbase.com/api/v3/brokerage/market/products";
const PUBLIC_PRODUCT_ENDPOINT: &str = "/api/v3/brokerage/market/products";
const PUBLIC_PRODUCTS_URL: &str = "https://api.coinbase.com/api/v3/brokerage/market/products";
const PUBLIC_PRODUCTS_ENDPOINT: &str = "/api/v3/brokerage/market/products";
const PUBLIC_PRODUCT_BOOK_URL: &str =
    "https://api.coinbase.com/api/v3/brokerage/market/product_book";
const PUBLIC_SERVER_TIME: &str = "https://api.coinbase.com/api/v3/brokerage/time";
const BEST_BID_ASK_URL: &str = "https://api.coinbase.com/api/v3/brokerage/best_bid_ask";
const BEST_BID_ASK_ENDPOINT: &str = "/api/v3/brokerage/best_bid_ask";
