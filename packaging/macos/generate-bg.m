#import <Cocoa/Cocoa.h>

static void drawBackgroundContents(void) {
    // Background subtle gradient
    NSGradient *bgGrad = [[NSGradient alloc] initWithStartingColor:[NSColor colorWithCalibratedRed:0.96 green:0.96 blue:0.98 alpha:1.0]
                                                       endingColor:[NSColor colorWithCalibratedRed:0.90 green:0.91 blue:0.93 alpha:1.0]];
    [bgGrad drawInRect:NSMakeRect(0, 0, 600, 400) angle:270];

    // Central rounded container card (symmetrically centered with 30pt margins: 540x340)
    NSBezierPath *card = [NSBezierPath bezierPathWithRoundedRect:NSMakeRect(30, 30, 540, 340) xRadius:16 yRadius:16];
    [[NSColor colorWithCalibratedWhite:1.0 alpha:0.82] setFill];
    [card fill];
    [[NSColor colorWithCalibratedWhite:0.82 alpha:0.75] setStroke];
    [card setLineWidth:1.0];
    [card stroke];

    // Directional arrow between rnazar (center X=160, Y=200) and Applications (center X=440, Y=200)
    // Symmetrically spans from X=246 to X=354 (length 108pt, exactly 26pt gap to each 120pt icon)
    CGFloat arrowY = 200.0;

    [NSGraphicsContext saveGraphicsState];
    NSShadow *arrowShadow = [[NSShadow alloc] init];
    [arrowShadow setShadowColor:[NSColor colorWithCalibratedRed:0.15 green:0.25 blue:0.45 alpha:0.15]];
    [arrowShadow setShadowOffset:NSMakeSize(0, -1)];
    [arrowShadow setShadowBlurRadius:3.0];
    [arrowShadow set];

    NSBezierPath *arrow = [NSBezierPath bezierPath];
    [arrow moveToPoint:NSMakePoint(246, arrowY)];
    [arrow lineToPoint:NSMakePoint(334, arrowY)];
    [[NSColor colorWithCalibratedRed:0.22 green:0.44 blue:0.80 alpha:0.85] setStroke];
    [arrow setLineWidth:4.0];
    [arrow setLineCapStyle:NSLineCapStyleRound];
    CGFloat dash[2] = {7.0, 5.0};
    [arrow setLineDash:dash count:2 phase:0.0];
    [arrow stroke];

    // Arrow head
    NSBezierPath *head = [NSBezierPath bezierPath];
    [head moveToPoint:NSMakePoint(330, arrowY + 11)];
    [head lineToPoint:NSMakePoint(354, arrowY)];
    [head lineToPoint:NSMakePoint(330, arrowY - 11)];
    [head closePath];
    [head setLineJoinStyle:NSLineJoinStyleRound];
    [[NSColor colorWithCalibratedRed:0.22 green:0.44 blue:0.80 alpha:0.90] setFill];
    [head fill];
    [NSGraphicsContext restoreGraphicsState];

    // Instruction text at bottom (Cocoa Y=56 -> Finder Y=344)
    NSString *text = @"Drag rnazar to Applications to install";
    NSDictionary *attrs = @{
        NSFontAttributeName: [NSFont systemFontOfSize:13 weight:NSFontWeightMedium],
        NSForegroundColorAttributeName: [NSColor colorWithCalibratedRed:0.38 green:0.44 blue:0.54 alpha:0.95]
    };
    NSSize textSize = [text sizeWithAttributes:attrs];
    [text drawAtPoint:NSMakePoint((600 - textSize.width) / 2.0, 56) withAttributes:attrs];

    // Header at top (Cocoa Y=328 -> Finder Y=72)
    NSString *title = @"rnazar — Evil Eye Ward & Crash Notifier";
    NSDictionary *titleAttrs = @{
        NSFontAttributeName: [NSFont systemFontOfSize:15 weight:NSFontWeightSemibold],
        NSForegroundColorAttributeName: [NSColor colorWithCalibratedRed:0.14 green:0.18 blue:0.26 alpha:0.95]
    };
    NSSize titleSize = [title sizeWithAttributes:titleAttrs];
    [title drawAtPoint:NSMakePoint((600 - titleSize.width) / 2.0, 328) withAttributes:titleAttrs];
}

static NSBitmapImageRep *renderBitmap(int scale) {
    int pixelsWide = 600 * scale;
    int pixelsHigh = 400 * scale;
    NSBitmapImageRep *rep = [[NSBitmapImageRep alloc]
        initWithBitmapDataPlanes:NULL
        pixelsWide:pixelsWide
        pixelsHigh:pixelsHigh
        bitsPerSample:8
        samplesPerPixel:4
        hasAlpha:YES
        isPlanar:NO
        colorSpaceName:NSDeviceRGBColorSpace
        bytesPerRow:pixelsWide * 4
        bitsPerPixel:32];

    [rep setSize:NSMakeSize(600, 400)];

    NSGraphicsContext *ctx = [NSGraphicsContext graphicsContextWithBitmapImageRep:rep];
    [NSGraphicsContext saveGraphicsState];
    [NSGraphicsContext setCurrentContext:ctx];

    drawBackgroundContents();

    [NSGraphicsContext restoreGraphicsState];
    return rep;
}

int main(int argc, char *argv[]) {
    @autoreleasepool {
        NSString *outDir = (argc > 1) ? [NSString stringWithUTF8String:argv[1]] : @"packaging/macos";
        BOOL isDir = NO;
        if (![[NSFileManager defaultManager] fileExistsAtPath:outDir isDirectory:&isDir] || !isDir) {
            // If argument is a file path, use its parent directory
            outDir = [outDir stringByDeletingLastPathComponent];
            if ([outDir length] == 0) outDir = @".";
        }

        NSString *png1xPath = [outDir stringByAppendingPathComponent:@"background.png"];
        NSString *png2xPath = [outDir stringByAppendingPathComponent:@"background@2x.png"];
        NSString *tiffPath = [outDir stringByAppendingPathComponent:@"background.tiff"];

        NSBitmapImageRep *rep1x = renderBitmap(1);
        NSBitmapImageRep *rep2x = renderBitmap(2);

        NSData *png1xData = [rep1x representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
        [png1xData writeToFile:png1xPath atomically:YES];

        NSData *png2xData = [rep2x representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
        [png2xData writeToFile:png2xPath atomically:YES];

        // Combine into multi-resolution TIFF for Finder Retina support
        NSString *cmd = [NSString stringWithFormat:@"tiffutil -cathidpicheck \"%@\" \"%@\" -out \"%@\"", png1xPath, png2xPath, tiffPath];
        int status = system([cmd UTF8String]);
        if (status != 0) {
            // Fallback: use NSImage TIFF representation
            NSImage *multiImage = [[NSImage alloc] initWithSize:NSMakeSize(600, 400)];
            [multiImage addRepresentation:rep1x];
            [multiImage addRepresentation:rep2x];
            NSData *tiffData = [multiImage TIFFRepresentation];
            [tiffData writeToFile:tiffPath atomically:YES];
        }

        NSLog(@"Successfully generated DMG background assets in %@", outDir);
    }
    return 0;
}

