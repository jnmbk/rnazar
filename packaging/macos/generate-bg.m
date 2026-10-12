#import <Cocoa/Cocoa.h>

int main(int argc, char *argv[]) {
    @autoreleasepool {
        int width = 1200;
        int height = 800; // 600x400 @ 2x Retina
        
        NSBitmapImageRep *rep = [[NSBitmapImageRep alloc]
            initWithBitmapDataPlanes:NULL
            pixelsWide:width
            pixelsHigh:height
            bitsPerSample:8
            samplesPerPixel:4
            hasAlpha:YES
            isPlanar:NO
            colorSpaceName:NSDeviceRGBColorSpace
            bytesPerRow:width * 4
            bitsPerPixel:32];
        
        [rep setSize:NSMakeSize(600, 400)];
        
        NSGraphicsContext *ctx = [NSGraphicsContext graphicsContextWithBitmapImageRep:rep];
        [NSGraphicsContext setCurrentContext:ctx];
        
        // Background subtle gradient
        NSGradient *bgGrad = [[NSGradient alloc] initWithStartingColor:[NSColor colorWithCalibratedRed:0.96 green:0.96 blue:0.98 alpha:1.0]
                                                           endingColor:[NSColor colorWithCalibratedRed:0.90 green:0.91 blue:0.93 alpha:1.0]];
        [bgGrad drawInRect:NSMakeRect(0, 0, 600, 400) angle:270];

        // Central rounded container card
        NSBezierPath *card = [NSBezierPath bezierPathWithRoundedRect:NSMakeRect(40, 40, 520, 320) xRadius:16 yRadius:16];
        [[NSColor colorWithCalibratedWhite:1.0 alpha:0.75] setFill];
        [card fill];
        [[NSColor colorWithCalibratedWhite:0.8 alpha:0.6] setStroke];
        [card setLineWidth:1.0];
        [card stroke];

        // Directional arrow between (240, 200) and (360, 200)
        CGFloat arrowY = 200;
        NSBezierPath *arrow = [NSBezierPath bezierPath];
        [arrow moveToPoint:NSMakePoint(240, arrowY)];
        [arrow lineToPoint:NSMakePoint(335, arrowY)];
        [[NSColor colorWithCalibratedRed:0.35 green:0.45 blue:0.65 alpha:0.85] setStroke];
        [arrow setLineWidth:4.0];
        [arrow setLineCapStyle:NSLineCapStyleRound];
        CGFloat dash[2] = {8.0, 5.0};
        [arrow setLineDash:dash count:2 phase:0.0];
        [arrow stroke];

        // Arrow head
        NSBezierPath *head = [NSBezierPath bezierPath];
        [head moveToPoint:NSMakePoint(330, arrowY + 12)];
        [head lineToPoint:NSMakePoint(355, arrowY)];
        [head lineToPoint:NSMakePoint(330, arrowY - 12)];
        [head closePath];
        [[NSColor colorWithCalibratedRed:0.35 green:0.45 blue:0.65 alpha:0.9] setFill];
        [head fill];

        // Instruction text at bottom
        NSString *text = @"Drag rnazar to Applications to install";
        NSDictionary *attrs = @{
            NSFontAttributeName: [NSFont systemFontOfSize:13 weight:NSFontWeightMedium],
            NSForegroundColorAttributeName: [NSColor colorWithCalibratedRed:0.35 green:0.40 blue:0.48 alpha:0.9]
        };
        NSSize textSize = [text sizeWithAttributes:attrs];
        [text drawAtPoint:NSMakePoint((600 - textSize.width) / 2.0, 70) withAttributes:attrs];

        // Header at top
        NSString *title = @"rnazar — Evil Eye Ward & Crash Notifier";
        NSDictionary *titleAttrs = @{
            NSFontAttributeName: [NSFont systemFontOfSize:15 weight:NSFontWeightSemibold],
            NSForegroundColorAttributeName: [NSColor colorWithCalibratedRed:0.15 green:0.20 blue:0.28 alpha:0.95]
        };
        NSSize titleSize = [title sizeWithAttributes:titleAttrs];
        [title drawAtPoint:NSMakePoint((600 - titleSize.width) / 2.0, 310) withAttributes:titleAttrs];

        NSData *pngData = [rep representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
        NSString *outPath = (argc > 1) ? [NSString stringWithUTF8String:argv[1]] : @"packaging/macos/background.png";
        [pngData writeToFile:outPath atomically:YES];
        NSLog(@"Successfully generated DMG background at %@", outPath);
    }
    return 0;
}

