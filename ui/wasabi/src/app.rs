use crate::alloc::string::ToString;
use crate::cursor::Cursor;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;
use noli::error::Result as OsResult;
use noli::prelude::SystemApi;
use noli::println;
use noli::rect::Rect;
use noli::sys::api::MouseEvent;
use noli::sys::wasabi::Api;
use noli::window::StringSize;
use noli::window::Window;
use saba_core::browser::Browser;
use saba_core::constants::*;
use saba_core::display_item::DisplayItem;
use saba_core::error::Error;
use saba_core::http::HttpResponse;
use saba_core::renderer::layout::computed_style::FontSize;
use saba_core::renderer::layout::computed_style::TextDecoration;

/// Struct representing a user interface using the WasabiOS.
#[derive(Debug)]
pub struct WasabiUI {
    browser: Rc<RefCell<Browser>>,
    input_url: String,
    input_mode: InputMode,
    window: Window,
    cursor: Cursor,
    window_position: (i64, i64),
}
impl WasabiUI {
    pub fn new(browser: Rc<RefCell<Browser>>, xy: (i64, i64), url: String) -> Self {
        Self {
            browser,
            input_url: url,
            input_mode: InputMode::Normal,
            window: Window::new(
                "saba".to_string(),
                WHITE,
                xy.0,
                xy.1,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            )
            .unwrap(),
            cursor: Cursor::new(),
            window_position: xy,
        }
    }
    fn setup_toolbar(&mut self) -> OsResult<()> {
        self.window
            .fill_rect(LIGHTGRAY, 0, 0, WINDOW_WIDTH, TOOL_BAR_HEIGHT)?;
        self.window
            .draw_line(GRAY, 0, TOOL_BAR_HEIGHT, WINDOW_WIDTH - 1, TOOL_BAR_HEIGHT)?;
        self.window.draw_line(
            DARKGRAY,
            0,
            TITLE_BAR_HEIGHT + 1,
            WINDOW_WIDTH - 1,
            TITLE_BAR_HEIGHT + 1,
        )?;
        self.window
            .draw_string(BLACK, 5, 5, "Address:", StringSize::Medium, false)?;
        self.window
            .fill_rect(WHITE, 70, 2, WINDOW_WIDTH - 74, 2 + ADDRESS_BAR_HEIGHT)?;
        self.window.draw_line(GRAY, 70, 2, WINDOW_WIDTH - 4, 2)?;
        self.window.draw_line(BLACK, 71, 3, WINDOW_WIDTH - 5, 3)?;
        self.window
            .draw_line(GRAY, 71, 3, 71, 1 + ADDRESS_BAR_HEIGHT)?;
        Ok(())
    }
    fn setup(&mut self) -> Result<(), Error> {
        if let Err(error) = self.setup_toolbar() {
            return Err(Error::InvalidUI(format!(
                "failed to initialize a toolbar with error: {:#?}",
                error
            )));
        }
        self.window.flush();
        Ok(())
    }
    pub fn start(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        self.setup()?;
        self.run_app(handle_url)?;
        Ok(())
    }
    fn run_app(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        if self.input_url != "".to_string() {
            self.update_address_bar()?;
            self.start_navigation(handle_url, self.input_url.clone())?;
        }
        loop {
            self.handle_key_input(handle_url)?;
            self.handle_mouse_input(handle_url)?;
        }
        Ok(())
    }
    fn handle_mouse_input(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        if let Some(MouseEvent { button, position }) = Api::get_mouse_cursor_info() {
            self.window.flush_area(self.cursor.rect());
            self.cursor.set_position(position.x, position.y);
            self.window.flush_area(self.cursor.rect());
            self.cursor.flush();
            if button.l() || button.c() || button.r() {
                let relative_pos = (
                    position.x - self.window_position.0,
                    position.y - self.window_position.1,
                );
                if relative_pos.0 < 0
                    || relative_pos.0 > WINDOW_WIDTH
                    || relative_pos.1 < 0
                    || relative_pos.1 > WINDOW_HEIGHT
                {
                    println!("button clicked OUTSIDE window: {button:?} {position:?}");
                    return Ok(());
                }
                if relative_pos.1 < TOOL_BAR_HEIGHT + TITLE_BAR_HEIGHT
                    && relative_pos.1 >= TITLE_BAR_HEIGHT
                {
                    self.clear_address_bar()?;
                    self.input_url = String::new();
                    self.input_mode = InputMode::Editing;
                    println!("button clicked in toolbar: {button:?} {position:?}");
                    return Ok(());
                }
                self.input_mode = InputMode::Normal;
                let position_in_content_area = (
                    relative_pos.0,
                    relative_pos.1 - TITLE_BAR_HEIGHT - TOOL_BAR_HEIGHT,
                );
                let page = self.browser.borrow().current_page();
                let next_destination = page.borrow_mut().clicked(position_in_content_area);
                if let Some(url) = next_destination {
                    self.input_url = url.clone();
                    self.update_address_bar()?;
                    self.start_navigation(handle_url, url)?;
                }
            }
        }
        Ok(())
    }
    fn handle_key_input(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        match self.input_mode {
            InputMode::Normal => {
                let _ = Api::read_key();
            }
            InputMode::Editing => {
                if let Some(c) = Api::read_key() {
                    if c == 0x0a as char {
                        self.start_navigation(handle_url, self.input_url.clone())?;
                        self.input_url = String::new();
                        self.input_mode = InputMode::Normal;
                    } else if c == 0x7f as char || c == 0x08 as char {
                        self.input_url.pop();
                    } else {
                        self.input_url.push(c);
                    }
                    self.update_address_bar()?;
                }
            }
        }
        Ok(())
    }
    fn update_address_bar(&mut self) -> Result<(), Error> {
        if self
            .window
            .fill_rect(WHITE, 72, 4, WINDOW_WIDTH - 76, ADDRESS_BAR_HEIGHT - 2)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to clear an address bar".to_string(),
            ));
        }
        if self
            .window
            .draw_string(BLACK, 74, 6, &self.input_url, StringSize::Medium, false)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to update an address bar".to_string(),
            ));
        }
        self.window.flush_area(
            Rect::new(
                self.window_position.0,
                self.window_position.1 + TITLE_BAR_HEIGHT,
                WINDOW_WIDTH,
                TITLE_BAR_HEIGHT,
            )
            .expect("failed to create a rect for the address bar"),
        );
        Ok(())
    }
    fn clear_address_bar(&mut self) -> Result<(), Error> {
        if self
            .window
            .fill_rect(WHITE, 72, 4, WINDOW_WIDTH - 76, ADDRESS_BAR_HEIGHT - 2)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to clear an address bar".to_string(),
            ));
        }
        self.window.flush_area(
            Rect::new(
                self.window_position.0,
                self.window_position.1 + TITLE_BAR_HEIGHT,
                WINDOW_WIDTH,
                TITLE_BAR_HEIGHT,
            )
            .expect("failed to create fot the address bar"),
        );
        Ok(())
    }
    fn start_navigation(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
        destination: String,
    ) -> Result<(), Error> {
        self.clear_content_area()?;
        match handle_url(destination) {
            Ok(response) => {
                let page = self.browser.borrow().current_page();
                page.borrow_mut().receive_response(response);
            }
            Err(e) => {
                return Err(e);
            }
        }
        self.update_ui()?;
        Ok(())
    }
    fn clear_content_area(&mut self) -> Result<(), Error> {
        if self
            .window
            .fill_rect(
                WHITE,
                0,
                TOOL_BAR_HEIGHT + 2,
                CONTENT_AREA_WIDTH,
                CONTENT_AREA_HEIGHT - 2,
            )
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to clear a content area".to_string(),
            ));
        }
        self.window.flush();
        Ok(())
    }
    fn update_ui(&mut self) -> Result<(), Error> {
        let display_items = self
            .browser
            .borrow()
            .current_page()
            .borrow()
            .display_items();
        for item in display_items {
            match item {
                DisplayItem::Text {
                    text,
                    style,
                    layout_point,
                } => {
                    if self
                        .window
                        .draw_string(
                            style.color().code_u32(),
                            layout_point.x() + WINDOW_PADDING,
                            layout_point.y() + WINDOW_PADDING + TOOL_BAR_HEIGHT,
                            &text,
                            convert_font_size(style.font_size()),
                            style.text_decoration() == TextDecoration::Underline,
                        )
                        .is_err()
                    {
                        return Err(Error::InvalidUI("failed to draw a string".to_string()));
                    }
                }
                DisplayItem::Rect {
                    style,
                    layout_point,
                    layout_size,
                } => {
                    if self
                        .window
                        .fill_rect(
                            style.background_color().code_u32(),
                            layout_point.x() + WINDOW_PADDING,
                            layout_point.y() + WINDOW_PADDING + TOOL_BAR_HEIGHT,
                            layout_size.width(),
                            layout_size.height(),
                        )
                        .is_err()
                    {
                        return Err(Error::InvalidUI("failed to draw a string".to_string()));
                    }
                }
                _ => {}
            }
        }
        self.window.flush();
        Ok(())
    }
}

/// Convert a `FontSize` to a `StringSize`.
/// # Parameters
/// - `size` - The `FontSize` to convert.
/// # Returns
/// - A `StringSize` representing the converted `FontSize`.
fn convert_font_size(size: FontSize) -> StringSize {
    match size {
        FontSize::Medium => StringSize::Medium,
        FontSize::XLarge => StringSize::Large,
        FontSize::XXLarge => StringSize::XLarge,
    }
}

/// Enum representing the current state of the input mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputMode {
    Normal,
    Editing,
}
