export async function downloadData(
  data: Uint8Array,
  fileName: string
): Promise<void> {
  if (window.kiosk) {
    const { canceled, filePath } = await window.kiosk.showSaveDialog({
      defaultPath: fileName,
    });

    if (canceled || !filePath) {
      return;
    }

    await window.kiosk.writeFile(filePath, data);
  } else {
    const blob = new Blob([data], { type: 'application/octet-stream' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = fileName;
    a.click();
  }
}
