use reqwest::Client;
use scraper::{Html, Selector};
use tokio;
use tokio::sync::Semaphore;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

async fn my_async_task(areaurl: String, semaphore: Arc<Semaphore>, client: Client) -> Result<(), Box<dyn std::error::Error>> {
    // 获取信号量许可，以限制并发请求数
    let _permit = semaphore.acquire().await.unwrap();
    
    // 设置延迟时间，这里设置为 2 秒，避免对目标服务器造成过大压力
    sleep(Duration::from_secs(2)).await;

    // 发送 HTTP GET 请求获取网页内容
    let area_response = client.get(areaurl).send().await?;
    
    // 获取响应的文本内容
    let area_body = area_response.text().await?;
    
    // 使用 `scraper` 库解析 HTML 文档
    let area_document = Html::parse_document(&area_body);
    
    // 定义选择器，用于提取地区名称
    let areaname_selector = Selector::parse(".search_default em")
        .map_err(|e| format!("Failed to parse selector: {:?}", e))?;  // 错误传播
    
    // 定义选择器，用于提取天气温度
    let areatem_selector = Selector::parse(".wea_weather.clearfix em")
        .map_err(|e| format!("Failed to parse selector: {:?}", e))?;  // 错误传播
    
    // 使用选择器从 HTML 文档中提取地区名称，并进行错误处理
    let areaname = area_document.select(&areaname_selector).next()
        .ok_or("Cannot find areaname")?
        .text()
        .collect::<String>();
    
    // 使用选择器从 HTML 文档中提取天气温度（这里使用 `expect` 进行错误处理）
    let areatem = area_document.select(&areatem_selector).next().expect("can not find tem").text().collect::<String>();

    // 打印地区名称和天气温度
    println!("{} {}", areaname, areatem);
    
    // 返回成功结果
    Ok(())
}
#[tokio::main]
async fn main() {
    let client = Client::new(); // 使用 Client 进行请求
    // let china_url="https://tianqi.moji.com/weather/china";
    // let china_response = reqwest::get(china_url).await.expect("Could not load url.");
    // let china_body = china_response.text().await.expect("No response body found.");
    // let china_document=Html::parse_document(&china_body);
    // let provinceurl_selector = Selector::parse(".city_list.clearfix a").unwrap();

    let provinces_and_municipalities: [&str; 34] = [
        "beijing",   // 北京
        "tianjin",   // 天津
        "shanghai",  // 上海
        "chongqing", // 重庆
        "hebei",     // 河北
        "shanxi",    // 山西
        "neimenggu", // 内蒙古
        "liaoning",  // 辽宁
        "jilin",     // 吉林
        "heilongjiang", // 黑龙江
        "jiangsu",   // 江苏
        "zhejiang",  // 浙江
        "anhui",     // 安徽
        "fujian",    // 福建
        "jiangxi",   // 江西
        "shandong",  // 山东
        "henan",     // 河南
        "hubei",     // 湖北
        "hunan",     // 湖南
        "guangdong", // 广东
        "guangxi",   // 广西
        "hainan",    // 海南
        "sichuan",   // 四川
        "guizhou",   // 贵州
        "yunnan",    // 云南
        "xizang",    // 西藏
        "shaanxi",    // 陕西
        "gansu",     // 甘肃
        "qinghai",   // 青海
        "ningxia",   // 宁夏
        "xinjiang",  // 新疆 s u a u a u a u a
        "taiwan",    // 台湾
        "hongkong",  // 香港
        "macau",     // 澳门
    ];
    let mut tasks = vec![];

    // 设置最大并发数
    let max_concurrent_requests = 5;
    let semaphore = Arc::new(Semaphore::new(max_concurrent_requests));

    for province in provinces_and_municipalities.iter(){
        // let provinceurl=provinceurl_ele.value().attr("href").expect("there is no href");
        // let parts: Vec<&str> = provinceurl.split('/').collect();
        // let province=parts.last().expect("can not see /");
        let province_url=format!("https://tianqi.moji.com/weather/china/{}/",province);
        let province_response = reqwest::get(province_url).await.expect("Could not load url.");
        let province_body = province_response.text().await.expect("No response body found.");
        let province_document=Html::parse_document(&province_body);
        let areaurl_selector = Selector::parse(".city_hot a").unwrap();
        for areaurl_ele in province_document.select(&areaurl_selector){
            let areaurl=areaurl_ele.value().attr("href").expect("there is no href").to_string();
            let areurl_clone=areaurl.clone();
            let client = client.clone(); // 克隆 Client 对象
            let semaphore = Arc::clone(&semaphore);
            // let task = tokio::spawn(my_async_task(areaurl, semaphore,client));  // 使用 `tokio::spawn` 启动异步任务
            let task = tokio::spawn(async move{
                if let Err(e) = my_async_task(areaurl, semaphore, client).await{
                    eprintln!("Task failed: {} {}", areurl_clone,e );
                }
            });
            tasks.push(task);
        }
    }
    // 等待所有任务完成
    futures::future::join_all(tasks).await;
    println!("Done");
}

