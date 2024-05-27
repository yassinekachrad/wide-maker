use reqwest::blocking::{Client, Response};
use reqwest::{self};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;

use chrono::Utc;

use super::bybit_auth::{_generate_get_signature, _generate_post_signature};


mod bybit_consts {
    pub const API_BASE_URL: &str = "https://api.bybit.com";
    pub const PLACE_ORDER_ENDPOINT: &str = "/v5/order/create";
    pub const CANCEL_ORDERS_ENDPOINT: &str = "/v5/order/cancel-all";
}

pub struct BybitClient {
    api_key: String,
    api_secret: String,
    client: Client,
}

impl BybitClient {
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            api_key,
            api_secret,
            client: Client::new(),
        }
    }

    pub fn place_order(&self, symbol: &str, price: &str, quantity: &str, side_is_buy: bool) -> Result<(), Box<dyn Error>> {
        let mut params = serde_json::Map::new();
        params.insert("category".to_string(), json!("linear"));
        params.insert("symbol".to_string(), json!(symbol));
        params.insert(
            "side".to_string(),
            json!(if side_is_buy { "Buy" } else { "Sell" }),
        );
        params.insert("positionIdx".to_string(), json!(0));
        params.insert("orderType".to_string(), json!("Limit"));
        params.insert("qty".to_string(), json!(quantity));
        params.insert("price".to_string(), json!(price));
        params.insert("timeInForce".to_string(), json!("GTC"));

        let response = self._send_post(bybit_consts::PLACE_ORDER_ENDPOINT, params);

        match response {
            Ok(response) => {
                println!("Response: {:?}", response.text()?);
                Ok(())
            }
            Err(error) => {
                println!("Error: {:?}", error);
                Err(error.into())
            }
        }
    }

    pub fn cancel_all_orders(&self, symbol: &str) -> Result<(), Box<dyn Error>> {
        let mut params = serde_json::Map::new();
        params.insert("category".to_string(), json!("linear"));
        params.insert("symbol".to_string(), json!(symbol));

        let response = self._send_post(bybit_consts::CANCEL_ORDERS_ENDPOINT, params);

        match response {
            Ok(response) => {
                println!("Response: {:?}", response.text()?);
                Ok(())
            }
            Err(error) => {
                println!("Error: {:?}", error);
                Err(error.into())
            }
        }
    }

    // Private

    fn _send_post(
        &self,
        endpoint: &str,
        params: serde_json::Map<String, Value>,
    ) -> reqwest::Result<Response> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let recv_window = "5000";
        let signature = _generate_post_signature(
            &timestamp,
            &self.api_key,
            recv_window,
            &params,
            &self.api_secret,
        )
        .unwrap();

        self.client
            .post(bybit_consts::API_BASE_URL.to_string() + endpoint)
            .json(&params)
            .header("X-BAPI-API-KEY", &self.api_key)
            .header("X-BAPI-SIGN", signature)
            .header("X-BAPI-SIGN-TYPE", "2")
            .header("X-BAPI-TIMESTAMP", timestamp.clone())
            .header("X-BAPI-RECV-WINDOW", recv_window)
            .header("Content-Type", "application/json")
            .send()
    }

    fn _send_get(&self, endpoint: &str, params: HashMap<&str, &str>) -> reqwest::Result<Response> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let recv_window = "5000";
        let signature = _generate_get_signature(
            &timestamp,
            &self.api_key,
            recv_window,
            &params,
            &self.api_secret,
        )
        .unwrap();

        self.client
            .get(bybit_consts::API_BASE_URL.to_string() + endpoint)
            .query(&params)
            .header("X-BAPI-API-KEY", &self.api_key)
            .header("X-BAPI-SIGN", signature)
            .header("X-BAPI-SIGN-TYPE", "2")
            .header("X-BAPI-TIMESTAMP", timestamp.clone())
            .header("X-BAPI-RECV-WINDOW", recv_window)
            .header("Content-Type", "application/json")
            .send()
    }
}

