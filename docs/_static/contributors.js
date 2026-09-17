document.addEventListener("DOMContentLoaded", () => {
    const list = document.querySelector(".sphinx-contributors_list");

    if (list) {
        list.innerHTML += list.innerHTML;
    }
});