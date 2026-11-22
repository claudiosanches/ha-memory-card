import { prepareZXingModule, readBarcodes } from 'zxing-wasm/reader';
import wasmUrl from 'zxing-wasm/reader/zxing_reader.wasm?url';
import type { Region } from '../api/types';

type Named = { name: string; abbreviation?: string | null };

// Served with the app, so scanning works without the CDN the library defaults to.
prepareZXingModule({
  overrides: { locateFile: (path: string, prefix: string) => (path.endsWith('.wasm') ? wasmUrl : prefix + path) },
});

/** The first UPC or EAN found in a photo, or null. */
export async function decodeBarcode(image: Blob): Promise<string | null> {
  return decode(image, true);
}

let frame: HTMLCanvasElement | undefined;

/** The UPC or EAN in the camera's current frame, or null. Quick, since it runs several times a second. */
export async function decodeVideoFrame(video: HTMLVideoElement): Promise<string | null> {
  if (video.readyState < HTMLMediaElement.HAVE_CURRENT_DATA || !video.videoWidth) return null;
  const scale = Math.min(1, 1280 / video.videoWidth);
  frame ??= document.createElement('canvas');
  frame.width = Math.round(video.videoWidth * scale);
  frame.height = Math.round(video.videoHeight * scale);
  const context = frame.getContext('2d', { willReadFrequently: true });
  if (!context) return null;
  context.drawImage(video, 0, 0, frame.width, frame.height);
  return decode(context.getImageData(0, 0, frame.width, frame.height), false);
}

async function decode(image: Blob | ImageData, tryHarder: boolean): Promise<string | null> {
  const results = await readBarcodes(image, {
    formats: ['EAN13', 'EAN8', 'UPCA', 'UPCE'],
    tryHarder,
    maxNumberOfSymbols: 1,
  });
  return results.find((result) => result.isValid)?.text ?? null;
}

/** A guess from the GS1 prefix: where the code was issued, which is usually where the box was sold. */
export function regionFromBarcode(code: string): Region | null {
  if (code.length === 12 || (code.length === 13 && code.startsWith('0'))) return 'na';
  const prefix = (n: number) => code.slice(0, n);
  if (['45', '49'].includes(prefix(2))) return 'jp';
  if (['789', '790'].includes(prefix(3))) return 'br';
  if (prefix(3) === '880') return 'kr';
  if (prefix(2) === '69') return 'cn';
  if (prefix(2) === '93') return 'au';
  if (prefix(2) === '94') return 'nz';
  if (/^(3[0-7]|4[0-4]|50|54|57|590|64|70|73|76|8[0-4]|87|9[01])/.test(code)) return 'eu';
  return null;
}

/** The platform a product title names, preferring the longest match. */
export function platformInTitle<P extends Named>(title: string, platforms: P[]): P | null {
  const lower = title.toLocaleLowerCase();
  const named = platforms.flatMap((platform) =>
    [platform.name, platform.abbreviation]
      .filter((name): name is string => !!name && name.length > 2)
      .filter((name) => new RegExp(`(^|[^a-z0-9])${escape(name.toLocaleLowerCase())}([^a-z0-9]|$)`).test(lower))
      .map((name) => ({ platform, length: name.length })),
  );
  named.sort((a, b) => b.length - a.length);
  return named[0]?.platform ?? null;
}

/** A product title without the platform and packaging words, ready for an IGDB search. */
export function gameTitle(title: string, platform: Named | null): string {
  let cleaned = title;
  // Only after a separator or at the end, so "Wii Sports" keeps its "Wii".
  for (const name of [platform?.name, platform?.abbreviation].filter((n): n is string => !!n)) {
    const words = escape(name);
    cleaned = cleaned
      .replace(new RegExp(`\\s*[-–|:]\\s*${words}\\b`, 'gi'), ' ')
      .replace(new RegExp(`[([]\\s*${words}\\s*[)\\]]`, 'gi'), ' ')
      .replace(new RegExp(`\\s+${words}\\s*$`, 'i'), '');
  }
  return cleaned
    .replace(/\b(standard|launch|day one|limited|collector'?s|deluxe) edition\b/gi, ' ')
    .replace(/[([][^)\]]*[)\]]/g, ' ')
    .replace(/\s+[-–|:]\s*$/g, '')
    .replace(/\s+[-–|]\s+(?=\s|$)/g, ' ')
    .replace(/\s{2,}/g, ' ')
    .replace(/^[\s\-–|:]+|[\s\-–|:]+$/g, '')
    .trim();
}

function escape(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
