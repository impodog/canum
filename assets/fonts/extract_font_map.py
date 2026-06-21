import json

import numpy as np
from PIL import Image


def extract_font_bounding_boxes(image_path, output_json_path):
    """
    Extract bounding boxes for printable ASCII characters from a font image.

    Args:
        image_path: Path to the font image file
        output_json_path: Path to save the JSON output
    """
    # Load image
    img = Image.open(image_path)
    img_array = np.array(img)

    # Get image dimensions
    height, width = img_array.shape[:2]

    # Determine grid width (number of columns)
    # Each cell is 11x11 pixels
    grid_width = width // 11
    grid_height = height // 11

    print(f"Image size: {width}x{height}")
    print(f"Grid dimensions: {grid_width} columns x {grid_height} rows")

    # Printable ASCII range: 32 (SPACE) to 126 (~)
    start_char = 32
    end_char = 126
    num_chars = end_char - start_char + 1  # 95 characters

    results = []

    for i in range(num_chars):
        char_code = start_char + i
        char = chr(char_code)

        # Calculate cell position in grid
        row = i // grid_width
        col = i % grid_width

        cell_x = col * 11
        cell_y = row * 11

        # Extract cell region
        cell = img_array[cell_y : cell_y + 11, cell_x : cell_x + 11]

        # Find non-white pixels
        # Handle both RGB and grayscale images
        if len(cell.shape) == 3:
            # RGB image - black is (0, 0, 0)
            non_black = np.any(cell != 255, axis=2)
        else:
            # Grayscale image - black is 0
            non_black = cell != 0

        # Find bounding box of non-black pixels
        coords = np.argwhere(non_black)

        if len(coords) > 0:
            y_min, x_min = coords.min(axis=0)
            y_max, x_max = coords.max(axis=0)

            # Convert to absolute coordinates in the image
            abs_x_min = cell_x + x_min
            abs_y_min = cell_y + y_min
            abs_x_max = cell_x + x_max
            abs_y_max = cell_y + y_max

            bbox = {
                "x": int(abs_x_min),
                "y": int(abs_y_min),
                "width": int(abs_x_max - abs_x_min + 1),
                "height": int(abs_y_max - abs_y_min + 1),
            }
        else:
            # Empty character (like SPACE)
            bbox = {"x": cell_x, "y": cell_y, "width": 0, "height": 0}

        results.append({"char": char, "ascii_code": char_code, "bbox": bbox})

    # Save to JSON
    with open(output_json_path, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2, ensure_ascii=False)

    print(f"Extracted {len(results)} character bounding boxes")
    print(f"Results saved to {output_json_path}")

    return results


# Example usage
if __name__ == "__main__":
    image_path = "normal.png"  # Replace with your font image path
    output_json_path = "normal.json"
    extract_font_bounding_boxes(image_path, output_json_path)
