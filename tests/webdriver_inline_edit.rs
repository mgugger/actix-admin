use tokio::test as tokio_test;
mod test_setup;
use test_setup::prelude::*;
use tokio;

mod webdriver_tests {
    use std::time::Duration;

    use super::*;
    use fantoccini::Locator;

    #[tokio_test]
    async fn webdriver_edit() -> Result<(), fantoccini::error::CmdError> {    
        let (server_task, geckodriver, c) = setup(true, true).await.unwrap();

        // Open the post list page
        c.goto("http://localhost:5555/admin/post/list").await?;
        let url = c.current_url().await?;
        assert_eq!(url.as_ref(), "http://localhost:5555/admin/post/list");

        // Click on edit first element
        let css_selector = "a[hx-get='/admin/post/edit/1']";
        c.find(Locator::Css(css_selector.into())).await?.click().await?;
        let url = c.current_url().await?;
        assert!(url.as_ref().contains("http://localhost:5555/admin/post/list"));

        // Fill the form
        let css_selector = "input[name='title']";
        let el = c.find(Locator::Css(css_selector.into())).await?;
        el.clear().await?;
        el.send_keys("testtitle").await?;
        let css_selector = "input[name='text']";
        let el = c.find(Locator::Css(css_selector.into())).await?;
        el.clear().await?;
        el.send_keys("testtext").await?;
        let css_selector = c.find(Locator::Css("select[name='tea_mandatory']")).await?;
        css_selector.select_by_value("EverydayTea").await?;
        let css_selector = "input[name='insert_date']";
        c.find(Locator::Css(css_selector.into())).await?.send_keys("2023-04-02").await?;

        // save
        let css_selector = "a[name='submitBtn']";
        c.find(Locator::Css(css_selector.into())).await?.click().await?;
        let url = c.current_url().await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        assert!(url.as_ref().contains("http://localhost:5555/admin/post/list"));

        // assert content of first row
        let table = c.find(Locator::Css("tbody")).await?;
        let table_rows = table.find_all(Locator::Css("tr")).await?;
        let row_text = table_rows[0].text().await?;
    
        let expected_texts = vec!["testtitle", "testtext", "2023-04-02", "EverydayTea"];
        for text in expected_texts {
            assert!(row_text.contains(text));
        }

        teardown(server_task, geckodriver, c).await
    }
}
