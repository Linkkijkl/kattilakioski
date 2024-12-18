const testApi = async () => {
    const response = await fetch("/api/hello");
    const body = document.querySelector("body");
    const h1 = document.createElement("h1");
    if (response.ok) {
        h1.textContent = await response.text();
    } else {
        h1.textContent = "api is not responsive :c";
    }
    body.appendChild(h1);
};

export { testApi };
