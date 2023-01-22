#[macro_use] extern crate rocket;
use rocket::response::content as content;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/html")]
fn html() -> content::RawHtml<&'static str> {
    content::RawHtml(r#"
        <title>ZSE Trader</title>
        <div align="center">
            <h1>ZSE Trader</h1>

            <p>Markets</p>
            <table style="border: 1px solid black">
                <thead>
                    <th>Market</th>
                    <th>Price</th>
                    <th>Volume</th>
                </thead>
                <tr>
                    <td>Market 1</td>
                    <td>100</td>
                    <td>1000</td>
                </tr>
                <tr>
                    <td>Market 2</td>
                    <td>200</td>
                    <td>2000</td>
                </tr>
            </table>
        </div>
    "#)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![html])
}
