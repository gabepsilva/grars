//! Windows-specific screenshot capture implementation

use std::env;
use std::process::Command;
use tracing::{debug, error, info};

/// PowerShell script for interactive screenshot region selection using Windows Forms.
/// This creates a translucent overlay and allows the user to draw a rectangle for capture.
/// Fixed to handle DPI scaling correctly by making the process DPI-aware.
const SCREENSHOT_PS_SCRIPT: &str = r#"
# Make the process DPI-aware to prevent Windows from auto-scaling
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class DPIHelper {
    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool SetProcessDPIAware();
}
"@ -Language CSharp
[DPIHelper]::SetProcessDPIAware()

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

# Get all screens for multi-monitor support (now returns physical pixels since DPI-aware)
$screens = [System.Windows.Forms.Screen]::AllScreens
$bounds = [System.Drawing.Rectangle]::Empty
foreach ($screen in $screens) {
    $bounds = [System.Drawing.Rectangle]::Union($bounds, $screen.Bounds)
}

# Create a full-screen bitmap
$bitmap = New-Object System.Drawing.Bitmap($bounds.Width, $bounds.Height)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)

# Capture the entire screen
$graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
$graphics.Dispose()

# Create a form for region selection
$form = New-Object System.Windows.Forms.Form
$form.FormBorderStyle = 'None'
$form.StartPosition = 'Manual'
$form.AutoScaleMode = 'None'
$form.Location = $bounds.Location
$form.Size = New-Object System.Drawing.Size($bounds.Width, $bounds.Height)
$form.TopMost = $true
$form.Cursor = [System.Windows.Forms.Cursors]::Cross

# Variables for selection
$script:startPoint = $null
$script:endPoint = $null
$script:selecting = $false
$script:cancelled = $false
$script:lastRect = $null

# Create a picture box for drawing the selection rectangle
$pictureBox = New-Object System.Windows.Forms.PictureBox
$pictureBox.Dock = 'Fill'
$pictureBox.SizeMode = 'Normal'
$pictureBox.Image = $bitmap.Clone()
# Enable double buffering for smoother rendering
$pictureBox.SetStyle(
    [System.Windows.Forms.ControlStyles]::OptimizedDoubleBuffer -bor
    [System.Windows.Forms.ControlStyles]::AllPaintingInWmPaint,
    $true
)
$form.Controls.Add($pictureBox)

$pictureBox.Add_MouseDown({
    param($sender, $e)
    if ($e.Button -eq 'Left') {
        # Clear any previous selection rectangle before starting new one
        if ($script:lastRect) {
            $clearRect = $script:lastRect
            $clearRect.Inflate(3, 3)
            $pictureBox.Invalidate($clearRect)
        }
        $script:startPoint = $e.Location
        $script:selecting = $true
        $script:lastRect = $null
    }
})

$pictureBox.Add_MouseMove({
    param($sender, $e)
    if ($script:selecting) {
        $script:endPoint = $e.Location
        $newRect = New-Object System.Drawing.Rectangle(
            [Math]::Min($script:startPoint.X, $script:endPoint.X),
            [Math]::Min($script:startPoint.Y, $script:endPoint.Y),
            [Math]::Abs($script:endPoint.X - $script:startPoint.X),
            [Math]::Abs($script:endPoint.Y - $script:startPoint.Y)
        )
        # Create union of old and new rectangles to ensure complete redraw
        # This prevents ghosting when resizing/shrinking the selection
        if ($script:lastRect) {
            $unionRect = [System.Drawing.Rectangle]::Union($script:lastRect, $newRect)
            # Add padding for pen width (2px) to ensure borders are fully cleared
            $unionRect.Inflate(3, 3)
            $pictureBox.Invalidate($unionRect)
        } else {
            # First draw - add padding for pen width
            $inflatedRect = $newRect
            $inflatedRect.Inflate(3, 3)
            $pictureBox.Invalidate($inflatedRect)
        }
        $script:lastRect = $newRect
    }
})

# Use Paint event for efficient drawing (no bitmap cloning)
$pictureBox.Add_Paint({
    param($sender, $e)
    if ($script:selecting -and $script:startPoint -and $script:endPoint) {
        $rect = New-Object System.Drawing.Rectangle(
            [Math]::Min($script:startPoint.X, $script:endPoint.X),
            [Math]::Min($script:startPoint.Y, $script:endPoint.Y),
            [Math]::Abs($script:endPoint.X - $script:startPoint.X),
            [Math]::Abs($script:endPoint.Y - $script:startPoint.Y)
        )
        # Use compositing mode to ensure clean drawing
        $e.Graphics.CompositingMode = [System.Drawing.Drawing2D.CompositingMode]::SourceOver
        # Draw selection overlay directly on the graphics context
        $pen = New-Object System.Drawing.Pen([System.Drawing.Color]::Red, 2)
        $brush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(50, 0, 120, 215))
        $e.Graphics.FillRectangle($brush, $rect)
        $e.Graphics.DrawRectangle($pen, $rect)
        $pen.Dispose()
        $brush.Dispose()
    }
})

$pictureBox.Add_MouseUp({
    param($sender, $e)
    if ($e.Button -eq 'Left' -and $script:selecting) {
        # endPoint already set in MouseMove, but set here for edge cases
        $script:endPoint = $e.Location
        $script:selecting = $false
        $form.Close()
    }
})

$form.Add_KeyDown({
    param($sender, $e)
    if ($e.KeyCode -eq 'Escape') {
        $script:cancelled = $true
        $form.Close()
    }
})

[System.Windows.Forms.Application]::Run($form)

if ($script:cancelled -or $script:startPoint -eq $null -or $script:endPoint -eq $null) {
    $bitmap.Dispose()
    exit 1
}

# Calculate selection rectangle
$x = [Math]::Min($script:startPoint.X, $script:endPoint.X)
$y = [Math]::Min($script:startPoint.Y, $script:endPoint.Y)
$width = [Math]::Abs($script:endPoint.X - $script:startPoint.X)
$height = [Math]::Abs($script:endPoint.Y - $script:startPoint.Y)

if ($width -lt 5 -or $height -lt 5) {
    $bitmap.Dispose()
    exit 1
}

# Crop the selection
$rect = New-Object System.Drawing.Rectangle($x, $y, $width, $height)
$cropped = $bitmap.Clone($rect, $bitmap.PixelFormat)
$bitmap.Dispose()

# Save to the output path (passed as argument)
$outputPath = $args[0]
$cropped.Save($outputPath, [System.Drawing.Imaging.ImageFormat]::Png)
$cropped.Dispose()

exit 0
"#;

/// Captures a screenshot region on Windows using PowerShell with Windows Forms.
pub(super) fn capture_region_windows() -> Result<String, String> {
    info!("Starting interactive screenshot region selection on Windows");
    
    let screenshot_path = env::temp_dir().join("insight-reader-screenshot.png");
    debug!(path = %screenshot_path.display(), "Screenshot will be saved to temp file");
    
    // Get the path as a string, properly escaped for PowerShell
    // Escape single quotes by doubling them (PowerShell escaping)
    let escaped_path = screenshot_path.to_string_lossy().replace('\'', "''");
    
    // Replace $args[0] placeholder in the script with the actual path
    // Use single quotes for literal string in PowerShell
    let script = SCREENSHOT_PS_SCRIPT.replace("$args[0]", &format!("'{}'", escaped_path));
    
    // Execute PowerShell script for region selection
    let output = match Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy", "Bypass",
            "-Command", &script,
        ])
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            error!(error = %e, "Failed to execute PowerShell screenshot command");
            return Err(format!("Failed to execute screenshot command: {}", e));
        }
    };
    
    // Check if the command succeeded
    if !output.status.success() {
        let exit_code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Exit code 1 typically means user cancelled (Escape key)
        if exit_code == 1 {
            debug!("User cancelled screenshot selection");
            return Err("Screenshot selection cancelled".to_string());
        }
        
        let error_msg = format!("Screenshot failed: {}", stderr.trim());
        error!(
            code = exit_code,
            stderr = %stderr.trim(),
            "PowerShell screenshot command failed"
        );
        return Err(error_msg);
    }
    
    // Verify the file was actually created
    if !screenshot_path.exists() {
        error!(path = %screenshot_path.display(), "Screenshot file was not created");
        return Err("Screenshot file was not created".to_string());
    }
    
    let path_str = screenshot_path.to_string_lossy().to_string();
    info!(path = %path_str, "Screenshot captured successfully");
    Ok(path_str)
}
