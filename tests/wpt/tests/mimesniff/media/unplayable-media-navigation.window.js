// META: title=Navigation to unplayable media
// META: variant=?file=undersized-ftyp.mp4
// META: variant=?file=ftyp-no-compatible-brands.mp4

// https://mimesniff.spec.whatwg.org/#matches-the-signature-for-mp4
// https://html.spec.whatwg.org/multipage/#navigate-media

const tests = {
  "vectors": [
    "undersized-ftyp.mp4",
    "ftyp-no-compatible-brands.mp4"
  ],
  "contentTypes": [
    "video/mp4"
  ]
};

// Add a variant above for each vector so crashes are reported per file.
const selectedFile = new URLSearchParams(location.search).get("file");

tests.vectors.forEach(vector => {
  if (selectedFile !== null && vector !== selectedFile) {
    return;
  }

  tests.contentTypes.forEach(type => {
    async_test(t => {
      const iframe = document.createElement("iframe");
      t.add_cleanup(() => iframe.remove());
      iframe.onload = t.step_func_done(() => {
        assert_equals(iframe.contentWindow.location.href, iframe.src,
                      "The iframe must finish navigating to the media response");
        assert_not_equals(iframe.contentDocument.querySelector("audio, video"), null,
                          "The navigation must produce a media document");
      });
      iframe.onerror = t.unreached_func("The media navigation failed");

      // An audio/video element alone does not exercise the navigation sniffer.
      iframe.src = `resources/${vector}?pipe=header(Content-Type,${type})`;
      document.body.append(iframe);
    }, `${vector} can be navigated to when served with Content-Type ${type}`);
  });
});
