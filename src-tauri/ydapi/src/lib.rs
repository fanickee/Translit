use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use chrono::Utc;
use md5;
use openssl::symm::{decrypt, Cipher};
use regex::Regex;
use reqwest::{header, Client, ClientBuilder};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum YdError {
    #[error("Ydapi has expired and needs to be updated")]
    ApiError(String),
    #[error("Ydapi Request Failed: `{0}`")]
    RequestError(#[from] reqwest::Error),
    #[error("unknown data store error")]
    Unknown,
}

#[derive(Debug)]
pub struct Ydapi {
    client: Client,
    domain: Vec<String>,
    webfanyi_key: String,
    decodekey: [u8; 16],
    decodeiv: [u8; 16],
    lang: Vec<(String, String)>
}

const YD_INDEX: &str = "https://fanyi.youdao.com/#/TextTranslate";
const YD_TRANS_KEY: &str = "https://dict.youdao.com/webtranslate/key";
const YD_KEY_GETTER: &str = "webfanyi-key-getter-2025";
const YD_DOMAIN_URL: &str = "https://doctrans-service.youdao.com/common/enums/list";
const YD_TRANSLATE_URL: &str = "https://dict.youdao.com/webtranslate";
impl Ydapi {
    pub async fn new() -> Result<Self, YdError> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Origin",
            header::HeaderValue::from_static("https://fanyi.youdao.com"),
        );
        headers.insert(
            "Referer",
            header::HeaderValue::from_static("https://fanyi.youdao.com/"),
        );
        let c = ClientBuilder::new()
            .cookie_store(true)
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36 Edg/114.0.1823.37")
            .default_headers(headers)
            .build().unwrap();
        let body = c.get(YD_INDEX).send().await?.text().await?;
        let js_regex = Regex::new(r#"src="(.+?/js/.+?.js)""#).unwrap();
        let js: Vec<_> = js_regex
            .captures_iter(&body)
            .filter_map(|s| {
                let s = s.get(1).unwrap().as_str();
                if s.find("js/app.").is_some() {
                    Some(s)
                } else {
                    return None;
                }
            })
            .collect();
        if js.len() == 0 {
            return Err(YdError::ApiError("can not find app js".into()));
        }
        let app = c.get(js[0]).send().await?.text().await?;
        /*  fetchTextTranslateSecretKey: async ({commit: e}, t) => {
                   const a = "webfanyi-key-getter-2025"
                     , n = "yU5nT5dK3eZ1pI4j";
                   return new Promise((t => {
                       o.A.getTextTranslateSecretKey({
                           keyid: a
                       }, n).then((a => {
                           0 === a.code && a.data.secretKey && (e("UPDATE_SECRET_KEY", a.data.secretKey),
                           e("UPDATE_DECODE_KEY", a.data.aesKey),
                           e("UPDATE_DECODE_IV", a.data.aesIv),
                           t(a.data.secretKey))
                       }
                       )).catch((e => {}
                       ))
                   }
                   ))
               }
        *
        */
        let tmp = Regex::new(format!(r#""{YD_KEY_GETTER}".+?"(.+?)""#).as_str())
            .unwrap()
            .captures(&app);
        if tmp.is_none() {
            return Err(YdError::ApiError("can not find key getter".into()));
        }
        let tmp = tmp.unwrap();
        let tmp = tmp.get(1).unwrap().as_str();
        let form: Vec<(String, String)> = Self::get_form(&YD_KEY_GETTER, tmp, None)
            .into_iter()
            .collect();
        let result: Value = c
            .get(YD_TRANS_KEY)
            .query(&form)
            .send()
            .await?
            .json()
            .await?;
        let webtranslate_key = result["data"]["secretKey"].as_str();
        if webtranslate_key.is_none() {
            return Err(YdError::ApiError("can not get web key".into()));
        }
        let webtranslate_key = webtranslate_key.unwrap();
        // get domain
        let domain_data: Value = c
            .get(YD_DOMAIN_URL)
            .query(&[("key", "domain".into()), ("_", Self::get_js_now_time())])
            .send()
            .await?
            .json()
            .await?;
        let tmp = domain_data["data"]
            .as_array()
            .ok_or(YdError::ApiError("can not get domain".into()))?;
        let domains: Vec<String> = tmp
            .iter()
            .map(|v| {
                v["msg"]
                    .as_str()
                    .ok_or(YdError::ApiError("can not get domain".into()))
                    .unwrap()
                    .to_string()
            })
            .collect();
        // get decode data key Iv
        // decodeKey: "ydsecret://query/key/B*RGygVywfNBwpmBaZg*WT7SIOUP2T0C9WHMZN39j^DAdaZhAnxvGcCY6VYFwnHl",
        // decodeIv: "ydsecret://query/iv/C@lZe2YzHtZ2CYgaXKSVfsb7Y4QWHjITPPZ0nQp87fBeJ!Iv6v^6fvi2WN@bYpJ4",
        let tmp = Regex::new(format!(r#"decodeKey:"(.+?)",decodeIv:"(.+?)""#).as_str())
            .unwrap()
            .captures(&app)
            .ok_or(YdError::ApiError("can not find key getter".into()))?;
        let decodekey = md5::compute(tmp.get(1).unwrap().as_str()).0;
        let decodeiv = md5::compute(tmp.get(2).unwrap().as_str()).0;

        let lang = Self::get_lang(&c).await?;

        return Ok(Self {
            client: c,
            webfanyi_key: webtranslate_key.to_string(),
            domain: domains,
            decodekey: decodekey,
            decodeiv: decodeiv,
            lang: lang
        });
    }

    fn get_js_now_time() -> String {
        Utc::now().timestamp_millis().to_string()
    }

    // function k(e, t) {
    //     const a = (new Date).getTime();
    //     return {
    //         sign: S(a, e),
    //         client: d,
    //         product: u,
    //         appVersion: m,
    //         vendor: h,
    //         pointParam: p,
    //         mysticTime: a,
    //         keyfrom: g,
    //         mid: b,
    //         screen: f,
    //         model: v,
    //         network: A,
    //         abtest: y,
    //         yduuid: t || "abcdefg"
    //     }
    // }
    fn get_form(keyid: &str, secret_key: &str, yduuid: Option<&str>) -> HashMap<String, String> {
        let mystic_time = Self::get_js_now_time();
        let yduuid = yduuid.unwrap_or("abcdefg");

        let sign_raw = format!(
            "client=fanyideskweb&mysticTime={}&product=webfanyi&key={}",
            mystic_time, secret_key
        );

        let sign = format!("{:x}", md5::compute(sign_raw));

        let mut map = HashMap::new();
        map.insert("keyid".into(), keyid.into());
        map.insert("sign".into(), sign);
        map.insert("client".into(), "fanyideskweb".into());
        map.insert("product".into(), "webfanyi".into());
        map.insert("appVersion".into(), "1.0.0".into());
        map.insert("vendor".into(), "web".into());
        map.insert("pointParam".into(), "client,mysticTime,product".into());
        map.insert("mysticTime".into(), mystic_time);
        map.insert("keyfrom".into(), "fanyi.web".into());
        map.insert("mid".into(), "1".into());
        map.insert("screen".into(), "1".into());
        map.insert("model".into(), "1".into());
        map.insert("network".into(), "wifi".into());
        map.insert("abtest".into(), "0".into());
        map.insert("yduuid".into(), yduuid.into());

        map
    }

    fn translate_dict_format(text: &str) -> String {
        let mut output = String::new();
        let result: Value = serde_json::from_str(text).unwrap();
        if result["code"] != 0 {
            output += &format!("code: {}\n注意只有通用场景支持中英", result["code"]);
            return output;
        }

        if let Some(array) = result["translateResult"].as_array() {
            for item in array {
                if let Some(sub_array) = item.as_array() {
                    for piece in sub_array {
                        if let Some(tgt) = piece.get("tgt") {
                            output.push_str(tgt.as_str().unwrap_or(""));
                        }
                    }
                }
            }
        }

        if let Some(ec) = result.get("dictResult").and_then(|d| d.get("ec")) {
            if let Some(word) = ec.get("word") {
                let usphone = word.get("usphone").and_then(|v| v.as_str()).unwrap_or("");
                let ukphone = word.get("ukphone").and_then(|v| v.as_str()).unwrap_or("");
                output.push('\n');
                output.push_str(&format!("{} {}\n", usphone, ukphone));

                if let Some(trs) = word.get("trs").and_then(|v| v.as_array()) {
                    for tr in trs {
                        let pos = tr.get("pos").and_then(|v| v.as_str()).unwrap_or("");
                        let tran = tr.get("tran").and_then(|v| v.as_str()).unwrap_or("");
                        output.push_str(&format!("{}{}\n", pos, tran));
                    }
                }

                if let Some(wfs) = word.get("wfs").and_then(|v| v.as_array()) {
                    for wf in wfs {
                        let name = wf["wf"]["name"].as_str().unwrap_or("");
                        let value = wf["wf"]["value"].as_str().unwrap_or("");
                        output.push_str(&format!("{}: {}\n", name, value));
                    }
                }
            }
        }
        output
    }
    pub fn domain(&self) -> &Vec<String> {
        return &self.domain;
    }
    pub async fn translate(
        &self,
        text: &str,
        domain_index: i32,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<String, YdError> {
        let mut form = Self::get_form("webfanyi", &self.webfanyi_key, None);
        form.insert("i".into(), text.into());
        form.insert("from".into(), from.unwrap_or("auto").into());
        form.insert("to".into(), to.unwrap_or("").into());
        form.insert("domain".into(), domain_index.to_string());
        form.insert("dictResult".into(), "true".into());
        let resp = self
            .client
            .post(YD_TRANSLATE_URL)
            .form(&form)
            .header("Accept", "application/json, text/plain, */*")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Host", "dict.youdao.com")
            .send()
            .await?
            .error_for_status()?;
        let data = resp.bytes().await?;
        let encode_text = URL_SAFE
            .decode(data)
            .map_err(|_| YdError::ApiError("decode base64 failed".into()))?;
        let cipher = Cipher::aes_128_cbc();
        let key = self.decodekey;
        let iv = self.decodeiv;
        let decode = decrypt(cipher, &key, Some(&iv), &encode_text)
            .map_err(|_| YdError::ApiError("decode 128 failed".into()))?;
        let decode_text = String::from_utf8_lossy(&decode);
        return Ok(Self::translate_dict_format(&decode_text));
    }

    async fn get_lang(client: &Client) -> Result<Vec<(String, String)>, YdError> {
        let mut list:Vec<(String, String)> = Vec::new();
        let json: Value = client.get("https://api-overmind.youdao.com/openapi/get/luna/dict/luna-front/prod/langType")
            .send().await?.json().await?;
        let com = json["data"]["value"]["textTranslate"]["specify"].as_array().ok_or(YdError::ApiError("lang data format error".into()))?;
        for c in com {
            list.push((c.get("code").unwrap().as_str().unwrap().into(), c.get("label").unwrap().as_str().unwrap().into()));
        }
        list.push(("auto".into(), "自动检测".into()));
        return Ok(list);
    }

    pub fn support_lang(&self) -> &Vec<(String, String)> {
        return &self.lang;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let result = Ydapi::new().await.unwrap();
        let text = result.translate("hello", 0, None, None).await.unwrap();
        println!("{}", text);
        assert!(text.find("你好").is_some());
    }
}
