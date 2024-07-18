/*
Cynthia Client-side script.

This script is embedded into any page Cynthia serves, always just before the closing </html>.
*/


setInterval(() => {
    const elements = document.getElementsByClassName('unparsedtimestamp');
    for (let i = elements.length - 1; i >= 0; i--) {
        const timestamp = Number.parseInt(elements[i].textContent);
        console.log("Parsing timestamp.");
        const jstimestamp = timestamp * 1000;
        const dateObject = new Date(jstimestamp);
        const data = dateObject.toLocaleString();
        elements[i].textContent = data.substring(0, data.length - 3);
        elements[i].classList.remove('unparsedtimestamp');
    }
}, 100);


function mobileorientation() {
    const csssays = getComputedStyle(document.body).getPropertyValue(
        "--screen-type-orientation"
    );
    if (csssays === "mobile") {
        return 1;
    }
    if (csssays === "landscape") {
        return 0;
    }
    console.error(
        `Could not determine 'mobilescreen()' from css value '${csssays}'.`
    );
}

if (document.getElementById("cynthiapageinfoshowdummyelem") != null) {
    let post_tags_htm = "";
    for (tag of pagemetainfo.tags) {
        if (tag !== pagemetainfo.tags[0]) {
            post_tags_htm = `${post_tags_htm}, <code class="post_tag">${tag}</code>`
        } else {
            post_tags_htm = `${post_tags_htm}<code class="post_tag">${tag}</code>`
        }
    };
    post_html = document.querySelector("main");
    post_html.innerHTML =
	`${post_html.innerHTML}
	<hr>
	<div id="taglist">
	<h4>Tags</h4>
	${post_tags_htm}
	</div>
	`
    post_tags = document.getElementsByClassName("post_tag");
    for (let i = post_tags.length - 1; i >= 0; i--) {
        const newpost_tag = document.createElement("a");
        newpost_tag.href = `/t/${post_tags.item(i).innerText}`;
        newpost_tag.innerHTML = `<code class='post_tag'>${post_tags.item(i).innerText}</code>`;
        post_tags.item(i).parentNode.replaceChild(newpost_tag, post_tags.item(i));
    }
    let pageinfosidebarelem = document.getElementById(
        "cynthiapageinfoshowdummyelem"
    );
    pageinfosidebarelem.setAttribute(
        "style",
        "opacity: 0.3; transition: all 2s ease-out 0s;"
    );
    pageinfosidebarelem.id = "pageinfosidebar";
    pageinfosidebarelem = document.getElementById("pageinfosidebar");
    let authorthumbnail = "";
    if (
        typeof pagemetainfo.author.thumbnail !== "undefined"
    ) {
        authorthumbnail = `<img alt="Author thumbnail" style="width: 2.5em" src="${pagemetainfo.author.thumbnail}">`;
    }
    let dates = "";
    if (typeof pagemetainfo.dates !== "undefined") {
        if (typeof pagemetainfo.dates.altered === "undefined" || pagemetainfo.dates.altered == null || pagemetainfo.dates.published === pagemetainfo.dates.altered) {
            dates = `<li>Posted: <span class="preparsedtimestamp">${(new Date((pagemetainfo.dates.published) * 1000).toLocaleString())}</span></li>`
        } else {
            dates = `
    <li>Posted: <span class="preparsedtimestamp">${(new Date((pagemetainfo.dates.published) * 1000).toLocaleString())}</span></li>
    <li>Edited: <span class="preparsedtimestamp">${(new Date((pagemetainfo.dates.altered) * 1000).toLocaleString())}</span></li>
    `
        }
    }
    pageinfosidebarelem.innerHTML = `
    <span class="not-on-mobile" style="position:absolute;right:0;top:0;font-size: 3em; cursor: pointer; " onclick="pageinfosidebar_rollup()">&#8665;</span>
    <p class="pageinfo-title">${pagemetainfo.title}</p>
    <ul>
      <li>Author: ${authorthumbnail} ${pagemetainfo.author.name}</li>
      ${dates}
      </ul>
    <p class="pageinfo-shortversion">${pagemetainfo.short}</p>
      `;

    function pageinfosidebar_rollup() {
        document.getElementById("pageinfosidebar").style.overflow = "hidden";
        document.getElementById("pageinfosidebar").style.width = "0";
        document.getElementById("pageinfosidebar").style.maxHeight = "310";
        document.getElementById("pageinfosidebartoggle").style.display = "";
        setTimeout(() => {
            document.getElementById("pageinfosidebartoggle").style.width = "40";
        }, 1700);
        setTimeout(() => {
            document.getElementById("pageinfosidebartoggle").style.padding = "8px";
        }, 1800);
    }

    function pageinfosidebar_rollout() {
        document.getElementById("pageinfosidebar").style.overflow = "";
        document.getElementById("pageinfosidebar").style.opacity = "100%";
        document.getElementById("pageinfosidebartoggle").style.overflow = "hidden";
        document.getElementById("pageinfosidebartoggle").style.width = "0";
        document.getElementById("pageinfosidebartoggle").style.padding = "0";
        setTimeout(() => {
            document.getElementById("pageinfosidebar").style.width = "";
        }, 1800);
        setTimeout(() => {
            document.getElementById("pageinfosidebar").style.height = "";
        }, 1900);
    }

    if (mobileorientation() || window.innerHeight < 350) {
        setTimeout(() => {
            pageinfosidebar_rollup();
        }, 6000);
        document.getElementById("pageinfosidebar").style.opacity = "100%";
    } else {
        document.getElementById("pageinfosidebar").style.opacity = "30%";
        document
            .getElementById("pageinfosidebar")
            .setAttribute("onmouseover", "this.style.opacity = '100%'");
        document
            .getElementById("pageinfosidebar")
            .setAttribute("onmouseout", "this.style.opacity = '30%'");
    }
}
