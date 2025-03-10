function injectInputs(w) {
  const keys = Object.keys(w);
  for (const i in keys) {
    const key = keys[i];
    if (key === 'user') {
      window.user = {
        id: 123456,
        role: "user",
        locations: {},
        gameLocationsCount: 0,
        trackedCategoryIds: [],
        suggestions: [],
        presets: [],

        ...(w.user || {}),
        
        hasPro: true,
      };
    } else {
      window[key] = w[key];
    }
  }

  const xhr = new XMLHttpRequest();
  xhr.open('GET', `/api/local/locations/${window.game.id}/${window.mapData.map.id}`, false);
  xhr.send();

  if (xhr.status === 200) {
    for (const location of JSON.parse(xhr.responseText)) {
      window.user.locations[location] = true;
    }
  }
}