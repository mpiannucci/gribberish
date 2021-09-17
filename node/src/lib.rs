use gribberish::message::Message;
use neon::{prelude::*, result::Throw};

struct GribMessage {
    inner: Message,
}

impl Finalize for GribMessage {}

impl GribMessage {
    fn get_var_name(mut cx: FunctionContext) -> JsResult<JsString> {
        let message = cx.this()
            .downcast_or_throw::<JsBox<GribMessage>, _>(&mut cx)?;
        
        let var_name = message
            .inner
            .variable_name()
            .or_else(|err| cx.throw_error(err))?;
        
        Ok(cx.string(var_name))
    }

    fn get_var_abbrev(mut cx: FunctionContext) -> JsResult<JsString> {
        let message = cx.this()
            .downcast_or_throw::<JsBox<GribMessage>, _>(&mut cx)?;
        
        let var_abbrev = message
            .inner
            .variable_abbrev()
            .or_else(|err| cx.throw_error(err))?;
        
        Ok(cx.string(var_abbrev))
    }

    fn get_units(mut cx: FunctionContext) -> JsResult<JsString> {
        let message = cx.this()
            .downcast_or_throw::<JsBox<GribMessage>, _>(&mut cx)?;
        
        let units = message
            .inner
            .unit()
            .or_else(|err| cx.throw_error(err))?;
        
        Ok(cx.string(units))
    }
}

fn parse_grib_message(mut cx: FunctionContext) -> JsResult<JsBox<GribMessage>> {
    let raw_js_data: Handle<JsBuffer> = cx.argument(0)?;
    let offset_js: Handle<JsNumber> = cx.argument(1)?;

    let guard = cx.lock();
    let js_data_slice = raw_js_data
        .borrow(&guard)
        .as_slice::<u8>();

    let mut raw_data: Vec<u8> = vec![0; js_data_slice.len()];
    raw_data.copy_from_slice(js_data_slice);

    let offset = offset_js.value(&mut cx) as usize;

    let message = Message::parse(&raw_data, offset).or_else(|err| cx.throw_error(err))?;

    Ok(cx.boxed(GribMessage { inner: message }))
}

fn parse_grib_messages(mut cx: FunctionContext) -> JsResult<JsArray> {
    let raw_js_data: Handle<JsBuffer> = cx.argument(0)?;

    let guard = cx.lock();
    let js_data_slice = raw_js_data
        .borrow(&guard)
        .as_slice::<u8>();

    let mut raw_data: Vec<u8> = vec![0; js_data_slice.len()];
    raw_data.copy_from_slice(js_data_slice);

    let messages = Message::parse_all(&raw_data);
    let arr = JsArray::new(&mut cx, messages.len() as u32);
    messages
        .into_iter()
        .map(|m| GribMessage {inner: m})
        .enumerate()
        .for_each(|g| {
            let boxed = cx.boxed(g.1).as_value(&mut cx);
            let _ = arr.set(&mut cx, g.0 as u32, boxed);
        }); 

    Ok(arr)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("parseGribMessage", parse_grib_message)?;
    cx.export_function("parseGribMessages", parse_grib_messages)?;
    
    cx.export_function("gribMessageGetVarName", GribMessage::get_var_name)?;
    cx.export_function("gribMessageGetVarAbbrev", GribMessage::get_var_abbrev)?;
    cx.export_function("gribMessageGetUnits", GribMessage::get_units)?;

    Ok(())
}
