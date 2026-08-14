use anyhow::Result;
use fake_user_agent::get_chrome_rua;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Serialize;
use sha2::{Digest, Sha256};



#[derive(Debug, Serialize)]
struct NewsArticle {
    source: String,
    url: String,
    title: String,
    author: Option<String>,
    published_at: Option<String>,
    content: String,
    hash: String,
}



fn hash_content(
    title: &str,
    content: &str
) -> String {

    let mut hasher = Sha256::new();

    hasher.update(
        format!(
            "{}{}",
            title,
            content
        )
    );


    hex::encode(
        hasher.finalize()
    )

}



fn extract_meta(
    document: &Html,
    selector: &str
) -> Option<String> {


    let selector =
        Selector::parse(selector)
        .ok()?;


    document
        .select(&selector)
        .next()
        .and_then(|x|
            x.value()
            .attr("content")
        )
        .map(|x|
            x.to_string()
        )

}




fn extract_author(
    document:&Html
)->Option<String>{


    let selectors=[

        "a[href*='author']",

        "a[href*='/authors/']",

        ".author-name",

        "[class*='author']"

    ];



    for s in selectors {


        if let Ok(selector)=
            Selector::parse(s)
        {


            if let Some(node)=
                document.select(&selector).next()
            {


                let text =
                    node.text()
                    .collect::<String>()
                    .trim()
                    .to_string();



                if text.len()>2 {

                    return Some(text);

                }

            }

        }

    }


    None

}




fn clean_content(
    text:String
)->String{


    let blacklist=[

        "ADVERTISEMENT",

        "Most Popular",

        "Sponsored",

        "Telegram Community",

        "Add as a preferred source"

    ];



    text
    .lines()

    .filter(|line|{


        let lower =
            line.to_lowercase();


        !blacklist
            .iter()
            .any(|x|
                lower.contains(
                    &x.to_lowercase()
                )
            )


    })


    .map(|x|
        x.trim()
    )


    .filter(|x|
        x.len()>30
    )


    .collect::<Vec<_>>()

    .join("\n\n")

}





fn extract_article_content(
    document:&Html
)->String{


    let containers=[


        "div.article-content",

        "div.article-body",

        "div.news-content",

        "article",

        "main"



    ];



    for container in containers {


        let selector =
            Selector::parse(container)
            .unwrap();



        if let Some(node)=
            document.select(&selector).next()
        {


            let p_selector =
                Selector::parse("p")
                .unwrap();



            let text =
                node
                .select(&p_selector)

                .map(|p|{


                    p.text()
                    .collect::<Vec<_>>()
                    .join(" ")

                })

                .collect::<Vec<_>>()

                .join("\n\n");



            let cleaned =
                clean_content(text);



            if cleaned.len()>200 {

                return cleaned;

            }

        }


    }


    String::new()

}






async fn scrape(
    url:&str
)->Result<NewsArticle>{



    let client =
        Client::builder()

        .user_agent(
            get_chrome_rua()
        )

        .build()?;




    let html =
        client

        .get(url)

        .send()

        .await?

        .text()

        .await?;





    let document =
        Html::parse_document(
            &html
        );



    let mut title =
        extract_meta(
            &document,
            "meta[property='og:title']"
        )
        .unwrap_or_default();



    let mut published =
        extract_meta(
            &document,
            "meta[property='article:published_time']"
        );



    let author =
        extract_author(
            &document
        );




    let mut content =
        extract_article_content(
            &document
        );




    /*
        JSON-LD fallback
    */


    let json_selector =
        Selector::parse(
            "script[type='application/ld+json']"
        )
        .unwrap();




    for node in document.select(&json_selector){


        let raw =
            node.text()
            .collect::<String>();



        if let Ok(json)=
            serde_json::from_str::<serde_json::Value>(
                &raw
            )
        {


            let obj =
                if json.is_array(){

                    &json[0]

                }else{

                    &json

                };



            if title.is_empty(){

                title =
                    obj["headline"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();

            }



            if published.is_none(){

                published =
                    obj["datePublished"]
                    .as_str()
                    .map(|x|
                        x.to_string()
                    );

            }



            if content.is_empty(){

                content =
                    obj["articleBody"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();

            }


        }


    }




    let hash =
        hash_content(
            &title,
            &content
        );




    Ok(
        NewsArticle{


            source:
                "investinglive"
                .to_string(),


            url:
                url.to_string(),


            title,


            author,


            published_at:
                published,


            content,


            hash,


        }
    )

}





#[tokio::main]
async fn main()->Result<()> {



    let url =
    "https://investinglive.com/news/swiss-economy-estimated-to-post-quarterly-growth-of-1-5-in-the-second-quarter/";



    let news =
        scrape(url)
        .await?;




    println!(
        "{}",
        serde_json::to_string_pretty(
            &news
        )?
    );



    Ok(())

}