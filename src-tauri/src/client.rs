use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, COOKIE, USER_AGENT};
use reqwest::blocking::Client;

fn get_headers(referer: &str, cookie: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Linux; Android 11; M2102J20SG) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.99 Mobile Safari/537.36 EdgA/97.0.1072.78"));
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static(
            "en,zh-HK;q=0.9,zh-TW;q=0.8,zh-CN;q=0.7,zh;q=0.6,en-GB;q=0.5,en-US;q=0.4",
        ),
    );
    headers.insert(ACCEPT, HeaderValue::from_static(r"text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"));
    headers.insert(COOKIE, HeaderValue::from_str(cookie).unwrap());
    headers.insert(
        "Referer",
        HeaderValue::from_str(&(referer.to_string() + "/novel/4353/250879.html")).unwrap(),
    );
    headers.insert(
        "accept-encoding",
        HeaderValue::from_static("gzip, deflate, br, zstd"),
    );
    headers.insert("priority", HeaderValue::from_static("u=0, i"));
    headers.insert(
        "sec-ch-ua",
        HeaderValue::from_static(
            "\"Microsoft Edge\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"",
        ),
    );
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?1"));
    headers.insert(
        "sec-ch-ua-platform",
        HeaderValue::from_static("\"Android\""),
    );
    headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
    headers.insert("upgrade-insecure-requests", HeaderValue::from_static("1"));
    headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers
}

pub fn get_client(referer: &str, cookie: &str) -> Client {
    // 创建一个 Cookie Jar
    let headers = get_headers(referer, cookie);
    // let jar = Arc::new(Jar::default());
    Client::builder().default_headers(headers).build().unwrap()
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Write};

    use super::*;

    #[test]
    fn image() {
        let client = get_client("https://tw.linovelib.com/", "night=1; Hm_lvt_1251eb70bc6856bd02196c68e198ee56=1742750685; Hm_lpvt_1251eb70bc6856bd02196c68e198ee56=1742750685; HMACCOUNT=2462002A73F462C9; _ga_NG72YQN6TX=GS1.1.1742750685.1.0.1742750685.0.0.0; _ga=GA1.1.273351663.1742750685; cf_clearance=nblxnZRmKyZVZPA8YS_q3V4wFzaDJVQvjBFnugYy7ZA-1742750601-1.2.1.1-vA1547Vr1ZpsDeNUNtBOmf9JExhoFxHal6kJdI58a5Sn8a9vKLtHdr.NrmtQ3dKqgDijqW_3GGxPcPEZyMZ4NFgu3uhiirR01QRySRx6fVW.7LEkV65O_EXuhqqCUFrPk8c.mHt7.YaCvHP2qGoyrhD79lhL0wv4OJdOvD2ikw6bz6d25504NtVXEspihjfUt_gOg9kr5UpFPCUOsE9_9yPi_Mu9pxUccreekjjenccueKFxW3StfgFoR7in.EeYfgMmevaRn3n4M5d4j7izmXGvWLxc_S2xgMLLTQU15_2ACAN4caZ2vlIcuECkxvbkY3XhAwjPdy2CVgKbeI4q.31BX6O8RXa8VFJrenTsGoh97OaVAw4ebTphPJo2dIdAioFZLAI81tMNAyw16RQpfPtMfJBaQc11JMx567ILJs0");
        let client = client.get("https://img3.readpai.com/0/23/110380/136701.jpg").header(
            ACCEPT,
            "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
        );
        let res = client.send().unwrap();
        let length = res.content_length().unwrap_or(0);
        println!("length: {}", length);
        File::create("test.jpg")
            .unwrap()
            .write_all(&res.bytes().unwrap())
            .unwrap();
    }
}