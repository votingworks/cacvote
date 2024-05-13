import { useState } from 'react';
import { PostScanScreen } from './screens/post_scan_screen';
import { ScanScreen } from './screens/scan_screen';

export function AppRoot(): JSX.Element {
  const [isScanning, setIsScanning] = useState(true);

  return isScanning ? (
    <ScanScreen onPostSuccess={() => setIsScanning(false)} />
  ) : (
    <PostScanScreen onScanAgainPressed={() => setIsScanning(true)} />
  );
}
