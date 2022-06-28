use gribberish::message::{Message, read_messages};
use neon::{prelude::*, result::Throw, types::JsDate};

struct GribMessage {
    inner: Message,
}

impl Finalize for GribMessage {}

impl GribMessage {
    fn get_var_name(mut cx: FunctionContext) -> JsResult<JsString> {
        let message = cx
            .this()
            .downcast_or_throw::<JsBox<GribMessage>, _>(&mut cx)?;

        let var_name = message
            .inner
            .variable_name()
            .or_else(|err| cx.throw_error(err))?;

        Ok(cx.string(var_name))
    }

    fn get_var_abbrev(mut cx: FunctionContext) -> JsResult<JsString> {
        let message = cx
            .this()
            .downcast_or_throw::<JsBox<GribMessage>, _>(&mut cx)?;

        let var_abbrev = message
            .inner
            .variable_abbrev()
            .or_else(|err| cx.throw_error(err))?;

        Ok(cx.string(var_abbrev))
    }

    fn get_units(mut cx: FunctionContext) -> JsResult<JsString> {
        let message = cx
            .this()
            .downcast_or_throw::<JsBox<GribMessage>, _>(&mut cx)?;

        let units = message.inner.unit().or_else(|err| cx.throw_error(err))?;

        Ok(cx.string(units))
    }

    fn get_array_index(mut cx: FunctionContext) -> JsResult<JsNumber> {
        let message = cx
            .this()
            .downcast_or_throw::<JsBox<GribMessage>, _>(&mut cx)?;

        let array_index = message
            .inner
            .array_index()
            .or_else(|err| cx.throw_error(err))?;

        match array_index {
            Some(array_index) => Ok(cx.number(array_index as u32)),
            None => Ok(cx.number(-1)),
        }
    }

    fn get_region(mut cx: FunctionContext) -> JsResult<JsObject> {
        let message = cx
            .this()
            .downcast_or_throw::<JsBox<GribMessage>, _>(&mut cx)?;

        let region = message
            .inner
            .location_region()
            .or_else(|err| cx.throw_error(err))?;

        let region_obj = cx.empty_object();

        let top_left_obj = cx.empty_object();
        let top_left_lat = cx.number(region.0 .0);
        let top_left_lon = cx.number(region.0 .1);
        top_left_obj.set(&mut cx, "lat", top_left_lat)?;
        top_left_obj.set(&mut cx, "lon", top_left_lon)?;
        region_obj.set(&mut cx, "topLeft", top_left_obj)?;

        let bottom_right_obj = cx.empty_object();
        let bottom_right_lat = cx.number(region.1 .0);
        let bottom_right_lon = cx.number(region.1 .1);
        bottom_right_obj.set(&mut cx, "lat", bottom_right_lat)?;
        bottom_right_obj.set(&mut cx, "lon", bottom_right_lon)?;
        region_obj.set(&mut cx, "bottomRight", bottom_right_obj)?;

        Ok(region_obj)
    }

    fn get_forecast_date(mut cx: FunctionContext) -> JsResult<JsDate> {
        let message = cx
            .this()
            .downcast_or_throw::<JsBox<GribMessage>, _>(&mut cx)?;

        let date = message.inner.forecast_date()
            .or_else(|err| cx.throw_error(err))?;
        
        let timestamp = date.timestamp_millis() as f64;
        JsDate::new(&mut cx, timestamp)
            .or_else(|err| cx.throw_error(err.to_string()))
    }

    fn get_reference_date(mut cx: FunctionContext) -> JsResult<JsDate> {
        let message = cx
            .this()
            .downcast_or_throw::<JsBox<GribMessage>, _>(&mut cx)?;

        let date = message.inner.reference_date()
            .or_else(|err| cx.throw_error(err))?;
        
        let timestamp = date.timestamp_millis() as f64;
        JsDate::new(&mut cx, timestamp)
            .or_else(|err| cx.throw_error(err.to_string()))
    }

    fn get_grid_shape(mut cx: FunctionContext) -> JsResult<JsObject> {
        let message = cx
            .this()
            .downcast_or_throw::<JsBox<GribMessage>, _>(&mut cx)?;

        let shape = message
            .inner
            .location_grid_dimensions()
            .or_else(|err| cx.throw_error(err))?;

        let shape_obj = cx.empty_object();
        let rows = cx.number(shape.0 as u32);
        let cols = cx.number(shape.1 as u32);
        shape_obj.set(&mut cx, "rows", rows)?;
        shape_obj.set(&mut cx, "cols", cols)?;

        Ok(shape_obj)
    }

    fn get_data(mut cx: FunctionContext) -> JsResult<JsArrayBuffer> {
        let message = cx
            .this()
            .downcast_or_throw::<JsBox<GribMessage>, _>(&mut cx)?;

        let data = message.inner.data().unwrap();

        let buffer_size = (data.len() as u32) * 8;

        let mut js_data = JsArrayBuffer::new(&mut cx, buffer_size)?;
        let guard = cx.lock();
        js_data
            .borrow_mut(&guard)
            .as_mut_slice::<f64>()
            .copy_from_slice(&data);

        Ok(js_data)
    }

    fn get_data_at_location(mut cx: FunctionContext) -> JsResult<JsNumber> {
        let message = cx
            .this()
            .downcast_or_throw::<JsBox<GribMessage>, _>(&mut cx)?;

        let lat = cx.argument::<JsNumber>(0)?.value(&mut cx);
        let lon = cx.argument::<JsNumber>(1)?.value(&mut cx);

        let data = message
            .inner
            .data_at_location(&(lat, lon))
            .unwrap_or(f64::NAN);

        Ok(cx.number(data))
    }

    fn get_location_data_index(mut cx: FunctionContext) -> JsResult<JsNumber> {
        let message = cx
            .this()
            .downcast_or_throw::<JsBox<GribMessage>, _>(&mut cx)?;

        let lat = cx.argument::<JsNumber>(0)?.value(&mut cx);
        let lon = cx.argument::<JsNumber>(1)?.value(&mut cx);

        let index = message
            .inner
            .data_index_for_location(&(lat, lon))
            .or_else(|err| cx.throw_error(err))?;

        Ok(cx.number(index as u32))
    }
}

fn parse_grib_message(mut cx: FunctionContext) -> JsResult<JsBox<GribMessage>> {
    let raw_js_data: Handle<JsBuffer> = cx.argument(0)?;
    let offset_js: Handle<JsNumber> = cx.argument(1)?;

    let guard = cx.lock();
    let js_data_slice = raw_js_data.borrow(&guard).as_slice::<u8>();

    let mut raw_data: Vec<u8> = vec![0; js_data_slice.len()];
    raw_data.copy_from_slice(js_data_slice);

    let offset = offset_js.value(&mut cx) as usize;

    let message = match Message::from_data(&raw_data, offset) {
        Some(m) => Ok(m), 
        None => cx.throw_error("Failed to read GribMessage"),
    }?;

    Ok(cx.boxed(GribMessage { inner: message }))
}

fn parse_grib_messages(mut cx: FunctionContext) -> JsResult<JsArray> {
    let raw_js_data: Handle<JsBuffer> = cx.argument(0)?;

    let guard = cx.lock();
    let js_data_slice = raw_js_data.borrow(&guard).as_slice::<u8>();

    let mut raw_data: Vec<u8> = vec![0; js_data_slice.len()];
    raw_data.copy_from_slice(js_data_slice);

    let messages = read_messages(raw_data.clone());
    let arr = JsArray::new(&mut cx, messages.count() as u32);

    let messages = read_messages(raw_data);
    messages
        .map(|m| GribMessage { inner: m })
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
    cx.export_function("gribMessageGetArrayIndex", GribMessage::get_array_index)?;
    cx.export_function("gribMessageGetRegion", GribMessage::get_region)?;
    cx.export_function("gribMessageGetGridShape", GribMessage::get_grid_shape)?;
    cx.export_function("gribMessageGetForecastDate", GribMessage::get_forecast_date)?;
    cx.export_function("gribMessageGetReferenceDate", GribMessage::get_reference_date)?;
    cx.export_function("gribMessageGetData", GribMessage::get_data)?;
    cx.export_function("gribMessageGetDataAtLocation", GribMessage::get_data_at_location)?;
    cx.export_function("gribMessageGetLocationDataIndex", GribMessage::get_location_data_index)?;

    Ok(())
}
