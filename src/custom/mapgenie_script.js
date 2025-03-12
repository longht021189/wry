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
  xhr.open('GET', `/api/local/map-data/${window.game.id}/${window.mapData.map.id}`, false);
  xhr.send();

  if (xhr.status === 200) {
    const response = JSON.parse(xhr.responseText);
    for (const location of response.locations) {
      window.user.locations[location] = true;
    }
  }
}