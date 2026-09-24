use image::{ColorType, DynamicImage, ImageError as NativeImageError, ImageFormat, ImageReader};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use yu_capability_image::{
    ImageEngine, ImageInfo, ImageOperationError, ResizeRequest, ResizeResult,
};

pub const ENGINE_ID: &str = "raster-rs";

#[derive(Debug, Default, Clone, Copy)]
pub struct RustImageEngine;

impl ImageEngine for RustImageEngine {
    fn id(&self) -> &'static str {
        ENGINE_ID
    }

    fn info(&self, path: &Path) -> Result<ImageInfo, ImageOperationError> {
        let (image, format) = open_image(path)?;
        let color = image.color();

        Ok(ImageInfo {
            path: path.to_string_lossy().into_owned(),
            format: format_name(format),
            width: image.width(),
            height: image.height(),
            color_type: format!("{color:?}").to_lowercase(),
            bit_depth: bits_per_channel(color),
            channels: color.channel_count(),
            has_alpha: color.has_alpha(),
        })
    }

    fn resize(&self, request: &ResizeRequest) -> Result<ResizeResult, ImageOperationError> {
        if request.output.exists() {
            return Err(ImageOperationError::output_conflict(format!(
                "output already exists: {}",
                request.output.display()
            )));
        }

        let output_format = ImageFormat::from_path(&request.output).map_err(|error| {
            ImageOperationError::unsupported(format!(
                "cannot infer a supported output format from {}: {error}",
                request.output.display()
            ))
        })?;

        let (image, _) = open_image(&request.input)?;
        let source_width = image.width();
        let source_height = image.height();
        let (width, height) =
            target_dimensions(source_width, source_height, request.width, request.height)?;

        let resized = image.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
        write_new_file(&resized, &request.output, output_format)?;

        Ok(ResizeResult {
            input: request.input.to_string_lossy().into_owned(),
            output: request.output.to_string_lossy().into_owned(),
            source_width,
            source_height,
            width,
            height,
            format: format_name(output_format),
        })
    }
}

fn open_image(path: &Path) -> Result<(DynamicImage, ImageFormat), ImageOperationError> {
    let reader = ImageReader::open(path).map_err(|error| {
        ImageOperationError::invalid_input(format!("cannot open {}: {error}", path.display()))
    })?;

    let reader = reader.with_guessed_format().map_err(|error| {
        ImageOperationError::invalid_input(format!(
            "cannot detect image format for {}: {error}",
            path.display()
        ))
    })?;

    let format = reader.format().ok_or_else(|| {
        ImageOperationError::unsupported(format!(
            "unsupported or unknown image format: {}",
            path.display()
        ))
    })?;

    let image = reader.decode().map_err(map_decode_error)?;
    Ok((image, format))
}

fn write_new_file(
    image: &DynamicImage,
    output: &Path,
    format: ImageFormat,
) -> Result<(), ImageOperationError> {
    let temporary = temporary_output_path(output);

    if let Err(error) = image.save_with_format(&temporary, format) {
        let _ = fs::remove_file(&temporary);
        return Err(map_encode_error(error, output));
    }

    if let Err(error) = fs::rename(&temporary, output) {
        let _ = fs::remove_file(&temporary);
        return Err(ImageOperationError::execution(format!(
            "cannot move completed image to {}: {error}",
            output.display()
        )));
    }

    Ok(())
}

fn temporary_output_path(output: &Path) -> PathBuf {
    let file_name = output
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_owned());

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();

    output.with_file_name(format!(
        ".{file_name}.yu-{}-{nonce}.tmp",
        std::process::id()
    ))
}

fn target_dimensions(
    source_width: u32,
    source_height: u32,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<(u32, u32), ImageOperationError> {
    if source_width == 0 || source_height == 0 {
        return Err(ImageOperationError::invalid_input(
            "source image has zero width or height",
        ));
    }

    if width == Some(0) || height == Some(0) {
        return Err(ImageOperationError::invalid_input(
            "resize dimensions must be greater than zero",
        ));
    }

    match (width, height) {
        (Some(width), Some(height)) => Ok((width, height)),
        (Some(width), None) => {
            let height = scaled_dimension(source_height, width, source_width)?;
            Ok((width, height))
        }
        (None, Some(height)) => {
            let width = scaled_dimension(source_width, height, source_height)?;
            Ok((width, height))
        }
        (None, None) => Err(ImageOperationError::invalid_input(
            "at least one of --width or --height is required",
        )),
    }
}

fn scaled_dimension(
    source_dimension: u32,
    target_dimension: u32,
    reference_dimension: u32,
) -> Result<u32, ImageOperationError> {
    let numerator = u64::from(source_dimension) * u64::from(target_dimension);
    let rounded = (numerator + u64::from(reference_dimension) / 2) / u64::from(reference_dimension);
    let rounded = rounded.max(1);

    u32::try_from(rounded).map_err(|_| {
        ImageOperationError::invalid_input("calculated resize dimension exceeds supported range")
    })
}

fn bits_per_channel(color: ColorType) -> u8 {
    let channels = u16::from(color.channel_count()).max(1);
    (color.bits_per_pixel() / channels) as u8
}

fn format_name(format: ImageFormat) -> String {
    format!("{format:?}").to_lowercase()
}

fn map_decode_error(error: NativeImageError) -> ImageOperationError {
    match error {
        NativeImageError::Unsupported(_) => {
            ImageOperationError::unsupported(format!("image format is not supported: {error}"))
        }
        _ => ImageOperationError::invalid_input(format!("cannot decode image: {error}")),
    }
}

fn map_encode_error(error: NativeImageError, output: &Path) -> ImageOperationError {
    match error {
        NativeImageError::Unsupported(_) => ImageOperationError::unsupported(format!(
            "output format is not supported for {}: {error}",
            output.display()
        )),
        _ => ImageOperationError::execution(format!(
            "cannot encode image to {}: {error}",
            output.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_only_preserves_aspect_ratio() {
        assert_eq!(
            target_dimensions(400, 200, Some(100), None).unwrap(),
            (100, 50)
        );
    }

    #[test]
    fn height_only_preserves_aspect_ratio() {
        assert_eq!(
            target_dimensions(400, 200, None, Some(50)).unwrap(),
            (100, 50)
        );
    }

    #[test]
    fn two_dimensions_are_exact() {
        assert_eq!(
            target_dimensions(400, 200, Some(123), Some(77)).unwrap(),
            (123, 77)
        );
    }

    #[test]
    fn missing_dimensions_are_rejected() {
        let error = target_dimensions(400, 200, None, None).unwrap_err();
        assert_eq!(
            error.kind,
            yu_capability_image::ImageErrorKind::InvalidInput
        );
    }
}
