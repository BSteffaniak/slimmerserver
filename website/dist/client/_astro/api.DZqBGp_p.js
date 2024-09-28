import{c as M,l as fe,n as ge,$ as S,y as X,z as Z,A as me,B as pe,C as $e}from"./web.o2PnP_jd.js";let $=[],ne=(e,n)=>{let t=[],i={get(){return i.lc||i.listen(()=>{})(),i.value},l:n||0,lc:0,listen(s,r){return i.lc=t.push(s,r||i.l)/2,()=>{let o=t.indexOf(s);~o&&(t.splice(o,2),--i.lc||i.off())}},notify(s){let r=!$.length;for(let o=0;o<t.length;o+=2)$.push(t[o],t[o+1],i.value,s);if(r){for(let o=0;o<$.length;o+=4){let a;for(let u=o+1;!a&&(u+=4)<$.length;)$[u]<$[o+1]&&(a=$.push($[o],$[o+1],$[o+2],$[o+3]));a||$[o]($[o+2],$[o+3])}$.length=0}},off(){},set(s){i.value!==s&&(i.value=s,i.notify())},subscribe(s,r){let o=i.listen(s,r);return s(i.value),o},value:e};return i};const he=5,x=6,N=10;let we=(e,n,t,i)=>(e.events=e.events||{},e.events[t+N]||(e.events[t+N]=i(s=>{e.events[t].reduceRight((r,o)=>(o(r),r),{shared:{},...s})})),e.events[t]=e.events[t]||[],e.events[t].push(n),()=>{let s=e.events[t],r=s.indexOf(n);s.splice(r,1),s.length||(delete e.events[t],e.events[t+N](),delete e.events[t+N])}),ye=1e3,be=(e,n)=>we(e,i=>{let s=n(i);s&&e.events[x].push(s)},he,i=>{let s=e.listen;e.listen=(...o)=>(!e.lc&&!e.active&&(e.active=!0,i()),s(...o));let r=e.off;return e.events[x]=[],e.off=()=>{r(),setTimeout(()=>{if(e.active&&!e.lc){e.active=!1;for(let o of e.events[x])o();e.events[x]=[]}},ye)},()=>{e.listen=s,e.off=r}}),K=e=>e,P={},J={addEventListener(){},removeEventListener(){}};function Ae(){try{return typeof localStorage<"u"}catch{return!1}}Ae()&&(P=localStorage);let Ie={addEventListener(e,n,t){window.addEventListener("storage",n),window.addEventListener("pageshow",t)},removeEventListener(e,n,t){window.removeEventListener("storage",n),window.removeEventListener("pageshow",t)}};typeof window<"u"&&(J=Ie);function ve(e,n=void 0,t={}){let i=t.encode||K,s=t.decode||K,r=ne(n),o=r.set;r.set=c=>{typeof c>"u"?delete P[e]:P[e]=i(c),o(c)};function a(c){c.key===e?c.newValue===null?o(void 0):o(s(c.newValue)):P[e]||o(void 0)}function u(){r.set(P[e]?s(P[e]):n)}return be(r,()=>{if(u(),t.listen!==!1)return J.addEventListener(e,a,u),()=>{J.removeEventListener(e,a,u)}}),r}class Se{_atom;_initial;_prevValue;_name;_listeners=[];constructor(n,t){this._initial=n,this._name=t,t?this._atom=ve(t,n,{encode:JSON.stringify,decode:JSON.parse}):this._atom=ne(n),this._prevValue=this.get()}get name(){return this._name}get initial(){return this._initial}get(){return this._atom.get()}set(n){this._atom.set(n),this._prevValue=n}listen(n){const t={baseListener:i=>n(i,this._prevValue),listener:n};this._atom.listen(t.baseListener),this._listeners.push(t)}off(n){const t=this._listeners.findIndex(i=>i.listener===n);t!==-1&&(this._listeners.splice(t,1),this._atom.off(),this._listeners.forEach(i=>{this._atom.listen(i.baseListener)}))}}function te(e,n){return new Se(e,n)}function Jn(e){let n=!0;const[t,i]=M(e.get(),{equals(r,o){return n?(n=!1,!1):r===o}}),s=r=>{i(r)};return fe(()=>{i(e.get()),e.listen(s)}),ge(()=>{e.off(s)}),[()=>{const r=n,o=t();return r?e.initial:o},r=>{e.set(r)}]}function ke(){let e=[];function n(r){return e.push(r),r}function t(r){return e.unshift(r),r}function i(r){e=e.filter(o=>o!==r)}return{on:n,onFirst:t,off:i,listeners:e,trigger:(...r)=>{for(const o of e)if(o(...r)===!1)break}}}function Vn(e,n){const t=Object.entries(e);return t.sort(([i],[s])=>{let r=n.indexOf(i),o=n.indexOf(s);return r=r===-1?n.length:r,o=o===-1?n.length:o,r-o}),t}class g{params;constructor(n){this.params=[],typeof n=="string"?(n[0]==="?"&&(n=n.substring(1)),n.trim().length>0&&n.split("&").map(t=>t.split("=")).forEach(([t,i])=>{this.params.push([t,i])})):n instanceof g?this.params.push(...n.params):n&&Object.entries(n).forEach(([t,i])=>{typeof i>"u"||this.params.push([t,i])})}get size(){return this.params.length}has(n){return!!this.params.find(([t,i])=>t===n)}get(n){const t=this.params.find(([i,s])=>i===n);if(t)return t[1]}set(n,t){const i=this.params.find(([s,r])=>s===n);i?i[1]=t:this.params.push([n,t])}delete(n){this.params=this.params.filter(([t,i])=>t!==n)}forEach(n){this.params.forEach(([t,i])=>n(t,i))}toString(){return`${this.params.map(([n,t])=>`${encodeURIComponent(n)}=${encodeURIComponent(t)}`).join("&")}`}}function Hn(e){return new URL(window.location.href).searchParams.get(e)}function Gn(e,n){const t=new URL(window.location.href);if(typeof n>"u")t.searchParams.delete(e);else{if(t.searchParams.get(e)===n){console.debug("Query param",e,"is already set");return}t.searchParams.set(e,n)}console.debug("Replacing url state with",t.toString()),window.history.replaceState({},"",t),window.Turbo.navigator.history.replace(t)}function Wn(){window.history.back()}function Xn(){return Te(navigator.userAgent||"opera"in window&&window.opera)}function Te(e){return e?/(android|bb\d+|meego).+mobile|avantgo|bada\/|blackberry|blazer|compal|elaine|fennec|hiptop|iemobile|ip(hone|od)|iris|kindle|lge |maemo|midp|mmp|mobile.+firefox|netfront|opera m(ob|in)i|palm( os)?|phone|p(ixi|re)\/|plucker|pocket|psp|series(4|6)0|symbian|treo|up\.(browser|link)|vodafone|wap|windows ce|xda|xiino/i.test(e)||/1207|6310|6590|3gso|4thp|50[1-6]i|770s|802s|a wa|abac|ac(er|oo|s-)|ai(ko|rn)|al(av|ca|co)|amoi|an(ex|ny|yw)|aptu|ar(ch|go)|as(te|us)|attw|au(di|-m|r |s )|avan|be(ck|ll|nq)|bi(lb|rd)|bl(ac|az)|br(e|v)w|bumb|bw-(n|u)|c55\/|capi|ccwa|cdm-|cell|chtm|cldc|cmd-|co(mp|nd)|craw|da(it|ll|ng)|dbte|dc-s|devi|dica|dmob|do(c|p)o|ds(12|-d)|el(49|ai)|em(l2|ul)|er(ic|k0)|esl8|ez([4-7]0|os|wa|ze)|fetc|fly(-|_)|g1 u|g560|gene|gf-5|g-mo|go(\.w|od)|gr(ad|un)|haie|hcit|hd-(m|p|t)|hei-|hi(pt|ta)|hp( i|ip)|hs-c|ht(c(-| |_|a|g|p|s|t)|tp)|hu(aw|tc)|i-(20|go|ma)|i230|iac( |-|\/)|ibro|idea|ig01|ikom|im1k|inno|ipaq|iris|ja(t|v)a|jbro|jemu|jigs|kddi|keji|kgt( |\/)|klon|kpt |kwc-|kyo(c|k)|le(no|xi)|lg( g|\/(k|l|u)|50|54|-[a-w])|libw|lynx|m1-w|m3ga|m50\/|ma(te|ui|xo)|mc(01|21|ca)|m-cr|me(rc|ri)|mi(o8|oa|ts)|mmef|mo(01|02|bi|de|do|t(-| |o|v)|zz)|mt(50|p1|v )|mwbp|mywa|n10[0-2]|n20[2-3]|n30(0|2)|n50(0|2|5)|n7(0(0|1)|10)|ne((c|m)-|on|tf|wf|wg|wt)|nok(6|i)|nzph|o2im|op(ti|wv)|oran|owg1|p800|pan(a|d|t)|pdxg|pg(13|-([1-8]|c))|phil|pire|pl(ay|uc)|pn-2|po(ck|rt|se)|prox|psio|pt-g|qa-a|qc(07|12|21|32|60|-[2-7]|i-)|qtek|r380|r600|raks|rim9|ro(ve|zo)|s55\/|sa(ge|ma|mm|ms|ny|va)|sc(01|h-|oo|p-)|sdk\/|se(c(-|0|1)|47|mc|nd|ri)|sgh-|shar|sie(-|m)|sk-0|sl(45|id)|sm(al|ar|b3|it|t5)|so(ft|ny)|sp(01|h-|v-|v )|sy(01|mb)|t2(18|50)|t6(00|10|18)|ta(gt|lk)|tcl-|tdg-|tel(i|m)|tim-|t-mo|to(pl|sh)|ts(70|m-|m3|m5)|tx-9|up(\.b|g1|si)|utst|v400|v750|veri|vi(rg|te)|vk(40|5[0-3]|-v)|vm40|voda|vulc|vx(52|53|60|61|70|80|81|83|85|98)|w3c(-| )|webc|whit|wi(g |nc|nw)|wmlb|wonu|x700|yas-|your|zeto|zte-/i.test(e.substring(0,4)):!1}function Kn(e,n){for(const t in e)if(!(t in n)||e[t]!==n[t])return!1;for(const t in n)if(!(t in e)||e[t]!==n[t])return!1;return!0}function Oe(e){return JSON.stringify(e,(()=>{const t=new WeakSet;return(i,s)=>{if(typeof s=="object"&&s!==null){if(t.has(s))return"[[circular]]";t.add(s)}return s}})())}function Pe(e){return typeof e=="string"?e:typeof e>"u"?"undefined":e===null?"null":typeof e=="object"?Oe(e):e.toString()}function Ue(e){throw new Error(e)}function Le(e,n){if(e===n)return!0;if(typeof e=="object"&&e!=null&&typeof n=="object"&&n!=null){if(Object.keys(e).length!=Object.keys(n).length)return!1;for(const t in e)if(n.hasOwnProperty(t)){if(!Le(e[t],n[t]))return!1}else return!1;return!0}else return!1}const Q=Symbol("store-raw"),L=Symbol("store-node"),I=Symbol("store-has"),ie=Symbol("store-self");function se(e){let n=e[S];if(!n&&(Object.defineProperty(e,S,{value:n=new Proxy(e,Ce)}),!Array.isArray(e))){const t=Object.keys(e),i=Object.getOwnPropertyDescriptors(e);for(let s=0,r=t.length;s<r;s++){const o=t[s];i[o].get&&Object.defineProperty(e,o,{enumerable:i[o].enumerable,get:i[o].get.bind(n)})}}return n}function A(e){let n;return e!=null&&typeof e=="object"&&(e[S]||!(n=Object.getPrototypeOf(e))||n===Object.prototype||Array.isArray(e))}function k(e,n=new Set){let t,i,s,r;if(t=e!=null&&e[Q])return t;if(!A(e)||n.has(e))return e;if(Array.isArray(e)){Object.isFrozen(e)?e=e.slice(0):n.add(e);for(let o=0,a=e.length;o<a;o++)s=e[o],(i=k(s,n))!==s&&(e[o]=i)}else{Object.isFrozen(e)?e=Object.assign({},e):n.add(e);const o=Object.keys(e),a=Object.getOwnPropertyDescriptors(e);for(let u=0,c=o.length;u<c;u++)r=o[u],!a[r].get&&(s=e[r],(i=k(s,n))!==s&&(e[r]=i))}return e}function R(e,n){let t=e[n];return t||Object.defineProperty(e,n,{value:t=Object.create(null)}),t}function q(e,n,t){if(e[n])return e[n];const[i,s]=M(t,{equals:!1,internal:!0});return i.$=s,e[n]=i}function _e(e,n){const t=Reflect.getOwnPropertyDescriptor(e,n);return!t||t.get||!t.configurable||n===S||n===L||(delete t.value,delete t.writable,t.get=()=>e[S][n]),t}function re(e){Z()&&q(R(e,L),ie)()}function Ee(e){return re(e),Reflect.ownKeys(e)}const Ce={get(e,n,t){if(n===Q)return e;if(n===S)return t;if(n===X)return re(e),t;const i=R(e,L),s=i[n];let r=s?s():e[n];if(n===L||n===I||n==="__proto__")return r;if(!s){const o=Object.getOwnPropertyDescriptor(e,n);Z()&&(typeof r!="function"||e.hasOwnProperty(n))&&!(o&&o.get)&&(r=q(i,n,r)())}return A(r)?se(r):r},has(e,n){return n===Q||n===S||n===X||n===L||n===I||n==="__proto__"?!0:(Z()&&q(R(e,I),n)(),n in e)},set(){return!0},deleteProperty(){return!0},ownKeys:Ee,getOwnPropertyDescriptor:_e};function h(e,n,t,i=!1){if(!i&&e[n]===t)return;const s=e[n],r=e.length;t===void 0?(delete e[n],e[I]&&e[I][n]&&s!==void 0&&e[I][n].$()):(e[n]=t,e[I]&&e[I][n]&&s===void 0&&e[I][n].$());let o=R(e,L),a;if((a=q(o,n,s))&&a.$(()=>t),Array.isArray(e)&&e.length!==r){for(let u=e.length;u<r;u++)(a=o[u])&&a.$();(a=q(o,"length",r))&&a.$(e.length)}(a=o[ie])&&a.$()}function oe(e,n){const t=Object.keys(n);for(let i=0;i<t.length;i+=1){const s=t[i];h(e,s,n[s])}}function qe(e,n){if(typeof n=="function"&&(n=n(e)),n=k(n),Array.isArray(n)){if(e===n)return;let t=0,i=n.length;for(;t<i;t++){const s=n[t];e[t]!==s&&h(e,t,s)}h(e,"length",i)}else oe(e,n)}function C(e,n,t=[]){let i,s=e;if(n.length>1){i=n.shift();const o=typeof i,a=Array.isArray(e);if(Array.isArray(i)){for(let u=0;u<i.length;u++)C(e,[i[u]].concat(n),t);return}else if(a&&o==="function"){for(let u=0;u<e.length;u++)i(e[u],u)&&C(e,[u].concat(n),t);return}else if(a&&o==="object"){const{from:u=0,to:c=e.length-1,by:l=1}=i;for(let m=u;m<=c;m+=l)C(e,[m].concat(n),t);return}else if(n.length>1){C(e[i],n,[i].concat(t));return}s=e[i],t=[i].concat(t)}let r=n[0];typeof r=="function"&&(r=r(s,t),r===s)||i===void 0&&r==null||(r=k(r),i===void 0||A(s)&&A(r)&&!Array.isArray(r)?oe(s,r):h(e,i,r))}function et(...[e,n]){const t=k(e||{}),i=Array.isArray(t),s=se(t);function r(...o){me(()=>{i&&o.length===1?qe(t,o[0]):C(t,o)})}return[s,r]}const V=Symbol("store-root");function U(e,n,t,i,s){const r=n[t];if(e===r)return;const o=Array.isArray(e);if(t!==V&&(!A(e)||!A(r)||o!==Array.isArray(r)||s&&e[s]!==r[s])){h(n,t,e);return}if(o){if(e.length&&r.length&&(!i||s&&e[0]&&e[0][s]!=null)){let c,l,m,w,y,v,B,T;for(m=0,w=Math.min(r.length,e.length);m<w&&(r[m]===e[m]||s&&r[m]&&e[m]&&r[m][s]===e[m][s]);m++)U(e[m],r,m,i,s);const E=new Array(e.length),z=new Map;for(w=r.length-1,y=e.length-1;w>=m&&y>=m&&(r[w]===e[y]||s&&r[m]&&e[m]&&r[w][s]===e[y][s]);w--,y--)E[y]=r[w];if(m>y||m>w){for(l=m;l<=y;l++)h(r,l,e[l]);for(;l<e.length;l++)h(r,l,E[l]),U(e[l],r,l,i,s);r.length>e.length&&h(r,"length",e.length);return}for(B=new Array(y+1),l=y;l>=m;l--)v=e[l],T=s&&v?v[s]:v,c=z.get(T),B[l]=c===void 0?-1:c,z.set(T,l);for(c=m;c<=w;c++)v=r[c],T=s&&v?v[s]:v,l=z.get(T),l!==void 0&&l!==-1&&(E[l]=r[c],l=B[l],z.set(T,l));for(l=m;l<e.length;l++)l in E?(h(r,l,E[l]),U(e[l],r,l,i,s)):h(r,l,e[l])}else for(let c=0,l=e.length;c<l;c++)U(e[c],r,c,i,s);r.length>e.length&&h(r,"length",e.length);return}const a=Object.keys(e);for(let c=0,l=a.length;c<l;c++)U(e[a[c]],r,a[c],i,s);const u=Object.keys(r);for(let c=0,l=u.length;c<l;c++)e[u[c]]===void 0&&h(r,u[c],void 0)}function ze(e,n={}){const{merge:t,key:i="id"}=n,s=k(e);return r=>{if(!A(r)||!A(s))return s;const o=U(s,{[V]:r},V,t,i);return o===void 0?r:o}}const j=new WeakMap,ae={get(e,n){if(n===Q)return e;const t=e[n];let i;return A(t)?j.get(t)||(j.set(t,i=new Proxy(t,ae)),i):t},set(e,n,t){return h(e,n,k(t)),!0},deleteProperty(e,n){return h(e,n,void 0,!0),!0}};function nt(e){return n=>{if(A(n)){let t;(t=j.get(n))||j.set(n,t=new Proxy(n,ae)),e(t)}return n}}var xe=(e=>typeof require<"u"?require:typeof Proxy<"u"?new Proxy(e,{get:(n,t)=>(typeof require<"u"?require:n)[t]}):e)(function(e){if(typeof require<"u")return require.apply(this,arguments);throw Error('Dynamic require of "'+e+'" is not supported')}),Ne=e=>(typeof e.clear=="function"||(e.clear=()=>{let n;for(;n=e.key(0);)e.removeItem(n)}),e),De=["domain","expires","path","secure","httpOnly","maxAge","sameSite"];function Qe(e){if(!e)return"";let n="";for(const t in e){if(!De.includes(t))continue;const i=e[t];n+=i instanceof Date?`; ${t}=${i.toUTCString()}`:typeof i=="boolean"?`; ${t}`:`; ${t}=${i}`}return n}function Re(e,n){return e.match(`(^|;)\\s*${n}\\s*=\\s*([^;]+)`)?.pop()??null}var ee;try{ee=xe("solid-start/server").useRequest}catch{ee=()=>(console.warn("It seems you attempt to use cookieStorage on the server without having solid-start installed or use vite."),{request:{headers:{get:()=>""}}})}var O=Ne({_read:()=>document.cookie,_write:(e,n,t)=>{document.cookie=`${e}=${n}${Qe(t)}`},getItem:(e,n)=>Re(O._read(n),e),setItem:(e,n,t)=>{O._write(e,n,t);{const s=Object.assign(new Event("storage"),{key:e,oldValue:null,newValue:n,url:globalThis.document.URL,storageArea:O});window.dispatchEvent(s)}},removeItem:(e,n)=>{O._write(e,"deleted",{...n,expires:new Date(0)})},key:(e,n)=>{let t=null,i=0;return O._read(n).replace(/(?:^|;)\s*(.+?)\s*=\s*[^;]+/g,(s,r)=>(!t&&r&&i++===e&&(t=r),"")),t},getLength:e=>{let n=0;return O._read(e).replace(/(?:^|;)\s*.+?\s*=\s*[^;]+/g,t=>(n+=t?1:0,"")),n},get length(){return this.getLength()}});function le(e,n={}){const t=n.storage||globalThis.localStorage;if(!t)return e;const i=n.storageOptions,s=n.name||`storage-${pe()}`,r=n.serialize||JSON.stringify.bind(JSON),o=n.deserialize||JSON.parse.bind(JSON),a=t.getItem(s,i),u=typeof e[0]=="function"?l=>e[1](()=>o(l)):l=>e[1](ze(o(l)));let c=!0;return a instanceof Promise?a.then(l=>c&&l&&u(l)):a&&u(a),[e[0],typeof e[0]=="function"?l=>{const m=e[1](l);return l?t.setItem(s,r(m),i):t.removeItem(s,i),c=!1,m}:(...l)=>{e[1](...l);const m=r($e(()=>e[0]));t.setItem(s,m,i),c=!1}]}function je(e){if(e)return typeof e=="number"||typeof e=="string"?e:"trackId"in e?e.trackId:"id"in e?e.id:void 0}function tt(e){return e.type==="LIBRARY"?{id:`${e.trackId}`,type:e.type,data:JSON.stringify(e)}:{id:`${e.id}`,type:e.type,data:JSON.stringify(e)}}var p;(e=>{const n=ke();e.onSignatureTokenUpdated=n.on,e.offSignatureTokenUpdated=n.off;const[t,i]=le(M("api.v2.signatureToken"),{name:"signatureToken"});function s(){return t()}e.signatureToken=s;function r(a){a!==t()&&(i(a),n.trigger(a))}e.setSignatureToken=r,(a=>{a.HOWLER="HOWLER"})(e.PlayerType||(e.PlayerType={})),(a=>{a.LOCAL="LOCAL",a.TIDAL="TIDAL",a.QOBUZ="QOBUZ",a.YT="YT"})(e.TrackSource||(e.TrackSource={})),e.AudioFormat={AAC:"AAC",FLAC:"FLAC",MP3:"MP3",OPUS:"OPUS",SOURCE:"SOURCE"};function o(a){a=a[0]==="/"?a.substring(1):a;const u=a.includes("?"),c=[],l=d(),m=l.clientId;l.clientId&&c.push(`clientId=${encodeURIComponent(m)}`);const w=e.signatureToken();w&&c.push(`signature=${encodeURIComponent(w)}`),l.staticToken&&c.push(`authorization=${encodeURIComponent(l.staticToken)}`);const y=`${u?"&":"?"}${c.join("&")}`;return`${l.apiUrl}/${a}${y}`}e.getPath=o,e.TrackAudioQuality={Low:"LOW",FlacLossless:"FLAC_LOSSLESS",FlacHiRes:"FLAC_HI_RES",FlacHighestRes:"FLAC_HIGHEST_RES"}})(p||(p={}));function it(e){const t=F.get().find(i=>i.id===e);if(!t)throw new Error(`Invalid connection id: ${e}`);Fe(e,t)}function Fe(e,n){const t=H.get(),i={id:e,name:n.name??t?.name??"",apiUrl:n.apiUrl??t?.apiUrl??"",clientId:n.clientId??t?.clientId??"",token:n.token??t?.token??"",staticToken:n.staticToken??t?.staticToken??""};H.set(i);const s=F.get(),r=s.findIndex(o=>o.id===i.id);r!==-1?s[r]=i:s.push(i),F.set([...s])}const F=te([],"api.v2.connections"),ce=()=>F.get(),H=te(ce()[0]??null,"api.v2.connection"),Me=()=>H.get();let G=1;ce()?.forEach(e=>{e.id>=G&&(G=e.id+1)});function st(){return G++}function d(){return Me()??Ue("No connection selected")}async function Be(e,n){const t=d(),i=new g({artistId:`${e}`});return await f(`${t.apiUrl}/menu/artist?${i}`,{credentials:"include",signal:n??null})}function Ye(e,n,t){if(!e)return"/img/album.svg";const i=e.type,s=new g({source:i,artistId:e.artistId?.toString()});switch(i){case"LIBRARY":if(e.containsCover)return p.getPath(`files/albums/${e.albumId}/${n}x${t}?${s}`);break;case"TIDAL":if(e.containsCover){if("albumId"in e)return p.getPath(`files/albums/${e.albumId}/${n}x${t}?${s}`);if("id"in e)return p.getPath(`files/albums/${e.id}/${n}x${t}?${s}`)}break;case"QOBUZ":if(e.containsCover){if("albumId"in e)return p.getPath(`files/albums/${e.albumId}/${n}x${t}?${s}`);if("id"in e)return p.getPath(`files/albums/${e.id}/${n}x${t}?${s}`)}break;case"YT":if(e.containsCover){if("albumId"in e)return p.getPath(`files/albums/${e.albumId}/${n}x${t}?${s}`);if("id"in e)return p.getPath(`files/albums/${e.id}/${n}x${t}?${s}`)}break}return"/img/album.svg"}function Ze(e){if(!e)return"/img/album.svg";const n=e.type,t=new g({source:n,artistId:e.artistId.toString()});switch(n){case"LIBRARY":if(e.containsCover)return p.getPath(`files/albums/${e.albumId}/source?${t}`);break;case"TIDAL":if(e.containsCover){if("albumId"in e)return p.getPath(`files/albums/${e.albumId}/source?${t}`);if("id"in e)return p.getPath(`files/albums/${e.id}/source?${t}`)}break;case"QOBUZ":if(e.containsCover){if("albumId"in e)return p.getPath(`files/albums/${e.albumId}/source?${t}`);if("id"in e)return p.getPath(`files/albums/${e.id}/source?${t}`)}break;case"YT":if(e.containsCover){if("albumId"in e)return p.getPath(`files/albums/${e.albumId}/source?${t}`);if("id"in e)return p.getPath(`files/albums/${e.id}/source?${t}`)}break}return"/img/album.svg"}async function Je(e,n){const t=d(),i=new g({albumId:`${e}`});return await f(`${t.apiUrl}/menu/album?${i}`,{credentials:"include",signal:n??null})}async function W(e=void 0,n){const t=d(),i=new g({artistId:e?.artistId?.toString(),tidalArtistId:e?.tidalArtistId?.toString(),qobuzArtistId:e?.qobuzArtistId?.toString(),offset:`${e?.offset??0}`,limit:`${e?.limit??100}`});return e?.sources&&i.set("sources",e.sources.join(",")),e?.sort&&i.set("sort",e.sort),e?.filters?.search&&i.set("search",e.filters.search),await f(`${t.apiUrl}/menu/albums?${i}`,{credentials:"include",signal:n??null})}async function Ve(e=void 0,n,t){let i=e?.offset??0,s=e?.limit??100;e=e??{offset:i,limit:s};const r=await W(e,t);let o=r.items;if(n?.(r.items,o,0),t?.aborted||!r.hasMore)return o;i=s,s=Math.min(Math.max(100,Math.ceil((r.total-s)/6)),1e3);const a=[];do a.push({...e,offset:i,limit:s}),i+=s;while(i<r.total);const u=[o,...a.map(()=>[])];return await Promise.all(a.map(async(c,l)=>{const m=await W(c,t);return u[l+1]=m.items,o=u.flat(),n?.(m.items,o,l+1),m})),o}function He(e,n,t){if(!e)return"/img/album.svg";const i=e.type,s=new g({source:i});switch(i){case"LIBRARY":if(e.containsCover)return p.getPath(`files/artists/${e.artistId}/${n}x${t}?${s}`);break;case"TIDAL":if(e.containsCover){if("artistId"in e)return p.getPath(`files/artists/${e.artistId}/${n}x${t}?${s}`);if("id"in e)return p.getPath(`files/artists/${e.id}/${n}x${t}?${s}`)}break;case"QOBUZ":if(e.containsCover){if("artistId"in e)return p.getPath(`files/artists/${e.artistId}/${n}x${t}?${s}`);if("id"in e)return p.getPath(`files/artists/${e.id}/${n}x${t}?${s}`)}break;case"YT":if(e.containsCover){if("artistId"in e)return p.getPath(`files/artists/${e.artistId}/${n}x${t}?${s}`);if("id"in e)return p.getPath(`files/artists/${e.id}/${n}x${t}?${s}`)}break}return"/img/album.svg"}function Ge(e){if(!e)return"/img/album.svg";const n=e.type,t=new g({source:n});switch(n){case"LIBRARY":if(e.containsCover)return p.getPath(`files/artists/${e.artistId}/source?${t}`);break;case"TIDAL":if(e.containsCover){if("artistId"in e)return p.getPath(`files/artists/${e.artistId}/source?${t}`);if("id"in e)return p.getPath(`files/artists/${e.id}/source?${t}`)}break;case"QOBUZ":if(e.containsCover){if("artistId"in e)return p.getPath(`files/artists/${e.artistId}/source?${t}`);if("id"in e)return p.getPath(`files/artists/${e.id}/source?${t}`)}break;case"YT":if(e.containsCover){if("artistId"in e)return p.getPath(`files/artists/${e.artistId}/source?${t}`);if("id"in e)return p.getPath(`files/artists/${e.id}/source?${t}`)}break}return"/img/album.svg"}async function We(e,n){const t=d();return await f(`${t.apiUrl}/menu/album/tracks?albumId=${e}`,{method:"GET",credentials:"include",signal:n??null})}async function Xe(e,n){const t=d();return await f(`${t.apiUrl}/menu/album/versions?albumId=${e}`,{method:"GET",credentials:"include",signal:n??null})}async function Ke(e,n){const t=d();return await f(`${t.apiUrl}/menu/tracks?trackIds=${e.join(",")}`,{method:"GET",credentials:"include",signal:n??null})}async function en(e=void 0,n){const t=d(),i=new g;return e?.sources&&i.set("sources",e.sources.join(",")),e?.sort&&i.set("sort",e.sort),e?.filters?.search&&i.set("search",e.filters.search),await f(`${t.apiUrl}/menu/artists?${i}`,{credentials:"include",signal:n??null})}async function nn(e){const n=d(),{token:t}=await f(`${n.apiUrl}/auth/signature-token`,{credentials:"include",method:"POST",signal:e??null});return t}const[tn,sn]=le(M([]),{name:"nonTunnelApis"});async function rn(e,n){const t=d(),i=tn();if(i.includes(t.apiUrl))return{notFound:!0};try{const{valid:s}=await f(`${t.apiUrl}/auth/validate-signature-token?signature=${e}`,{credentials:"include",method:"POST",signal:n??null});return{valid:!!s}}catch(s){return s instanceof de&&s.response.status===404?(sn([...i,t.apiUrl]),{notFound:!0}):{valid:!1}}}async function on(){console.debug("Refetching signature token");const e=await b.fetchSignatureToken();e?p.setSignatureToken(e):console.error("Failed to fetch signature token")}async function an(){if(!d().token)return;const n=p.signatureToken();if(!n){await b.refetchSignatureToken();return}const{valid:t,notFound:i}=await b.validateSignatureTokenAndClient(n);if(i){console.debug("Not hitting tunnel server");return}t||await b.refetchSignatureToken()}async function ln(e,n){const t=d();try{return await f(`${t.apiUrl}/auth/magic-token?magicToken=${e}`,{credentials:"include",signal:n??null})}catch{return!1}}async function ue(e,n,t,i){const s=d(),r=new g({query:e,offset:n?.toString()??void 0,limit:t?.toString()??void 0});return await f(`${s.apiUrl}/search/global-search?${r.toString()}`,{credentials:"include",signal:i??null})}async function D(e,n,t,i,s){const r=d(),o=new g({query:e,offset:t?.toString()??void 0,limit:i?.toString()??void 0});return await f(`${r.apiUrl}/${n}/search?${o.toString()}`,{credentials:"include",signal:s??null})}async function cn(e,n,t,i,s){const r=[];return await Promise.all([(async()=>{const o=(await ue(e,n,t,s)).results;r.push(...r),i?.(o,r,"LIBRARY")})(),(async()=>{const o=(await D(e,"tidal",n,t,s)).results;r.push(...r),i?.(o,r,"TIDAL")})(),(async()=>{const o=(await D(e,"qobuz",n,t,s)).results;r.push(...r),i?.(o,r,"QOBUZ")})(),(async()=>{const o=(await D(e,"yt",n,t,s)).results;r.push(...r),i?.(o,r,"YT")})()]),r}async function un(e,n){const t=d(),i=new g({tidalArtistId:`${e}`});return await f(`${t.apiUrl}/menu/artist?${i}`,{credentials:"include",signal:n??null})}async function dn(e,n){const t=d(),i=new g({qobuzArtistId:`${e}`});return await f(`${t.apiUrl}/menu/artist?${i}`,{credentials:"include",signal:n??null})}async function fn(e,n){const t=d(),i=new g({tidalAlbumId:`${e}`});return await f(`${t.apiUrl}/menu/artist?${i}`,{credentials:"include",signal:n??null})}async function gn(e,n){const t=d(),i=new g({artistId:`${e}`});return await f(`${t.apiUrl}/tidal/artists?${i}`,{credentials:"include",signal:n??null})}async function mn(e,n){const t=d(),i=new g({artistId:`${e}`});return await f(`${t.apiUrl}/qobuz/artists?${i}`,{credentials:"include",signal:n??null})}function _(e){return e.toSorted((n,t)=>n.dateReleased?t.dateReleased?t.dateReleased.localeCompare(n.dateReleased):-1:1)}async function pn(e,n,t,i){const s={lps:[],epsAndSingles:[],compilations:[]},r=[];return(!t||t.find(o=>o==="LP"))&&r.push((async()=>{const o=await b.getTidalArtistAlbums(e,"LP",i??null);if(s.lps=o.items,n){const{lps:a,epsAndSingles:u,compilations:c}=s;n(_([...a,...u,...c]))}})()),(!t||t.find(o=>o==="EPS_AND_SINGLES"))&&r.push((async()=>{const o=await b.getTidalArtistAlbums(e,"EPS_AND_SINGLES",i??null);if(n){s.epsAndSingles=o.items;const{lps:a,epsAndSingles:u,compilations:c}=s;n(_([...a,...u,...c]))}})()),(!t||t.find(o=>o==="COMPILATIONS"))&&r.push((async()=>{const o=await b.getTidalArtistAlbums(e,"COMPILATIONS",i??null);if(n){s.compilations=o.items;const{lps:a,epsAndSingles:u,compilations:c}=s;n(_([...a,...u,...c]))}})()),await Promise.all(r),s}async function $n(e,n,t,i){const s={lps:[],epsAndSingles:[],compilations:[]},r=[];return(!t||t.find(o=>o==="LP"))&&r.push((async()=>{const o=await b.getQobuzArtistAlbums(e,"LP",i??null);if(s.lps=o.items,n){const{lps:a,epsAndSingles:u,compilations:c}=s;n(_([...a,...u,...c]))}})()),(!t||t.find(o=>o==="EPS_AND_SINGLES"))&&r.push((async()=>{const o=await b.getQobuzArtistAlbums(e,"EPS_AND_SINGLES",i??null);if(n){s.epsAndSingles=o.items;const{lps:a,epsAndSingles:u,compilations:c}=s;n(_([...a,...u,...c]))}})()),(!t||t.find(o=>o==="COMPILATIONS"))&&r.push((async()=>{const o=await b.getQobuzArtistAlbums(e,"COMPILATIONS",i??null);if(n){s.compilations=o.items;const{lps:a,epsAndSingles:u,compilations:c}=s;n(_([...a,...u,...c]))}})()),await Promise.all(r),s}async function hn(e,n,t){const i=d(),s=new g({artistId:`${e}`});return n&&s.set("albumType",n),await f(`${i.apiUrl}/tidal/artists/albums?${s}`,{credentials:"include",signal:t??null})}async function wn(e,n,t){const i=d(),s=new g({artistId:`${e}`});return n&&s.set("releaseType",n),await f(`${i.apiUrl}/qobuz/artists/albums?${s}`,{credentials:"include",signal:t??null})}async function yn(e,n){const t=d(),i=new g({tidalAlbumId:`${e}`});return await f(`${t.apiUrl}/menu/album?${i}`,{credentials:"include",signal:n??null})}async function bn(e,n){const t=d(),i=new g({qobuzAlbumId:`${e}`});return await f(`${t.apiUrl}/menu/album?${i}`,{credentials:"include",signal:n??null})}async function An(e,n){const t=d(),i=new g({tidalArtistId:`${e}`});return await f(`${t.apiUrl}/menu/albums?${i}`,{credentials:"include",signal:n??null})}async function In(e,n){const t=d(),i=new g({qobuzArtistId:`${e}`});return await f(`${t.apiUrl}/menu/albums?${i}`,{credentials:"include",signal:n??null})}async function vn(e,n){const t=d(),i=new g({albumId:`${e}`});return await f(`${t.apiUrl}/tidal/albums?${i}`,{credentials:"include",signal:n??null})}async function Sn(e,n){const t=d(),i=new g({albumId:`${e}`});return await f(`${t.apiUrl}/qobuz/albums?${i}`,{credentials:"include",signal:n??null})}async function kn(e,n){const t=d(),i=new g({albumId:`${e}`});return await f(`${t.apiUrl}/tidal/albums/tracks?${i}`,{credentials:"include",signal:n??null})}async function Tn(e,n){const t=d(),i=new g({albumId:`${e}`});return await f(`${t.apiUrl}/qobuz/albums/tracks?${i}`,{credentials:"include",signal:n??null})}async function On(e,n){const t=d(),i=new g({albumId:`${e}`});return await f(`${t.apiUrl}/yt/albums/tracks?${i}`,{credentials:"include",signal:n??null})}async function Pn(e,n){const t=d(),i=new g({trackId:`${e}`});return await f(`${t.apiUrl}/tidal/track?${i}`,{credentials:"include",signal:n??null})}async function Un(e,n,t,i){const s=d(),r=new g({audioQuality:t,trackId:`${e}`,source:`${n}`});return(await f(`${s.apiUrl}/files/tracks/url?${r}`,{credentials:"include",signal:i??null}))[0]}async function Ln(e,n){const t=d(),i=new g({albumId:e.tidalAlbumId?.toString()??e.qobuzAlbumId,source:e.tidalAlbumId?"TIDAL":e.qobuzAlbumId?"QOBUZ":void 0});return await f(`${t.apiUrl}/menu/album?${i}`,{method:"POST",credentials:"include",signal:n??null})}async function _n(e,n){const t=d(),i=new g({albumId:e.tidalAlbumId?.toString()??e.qobuzAlbumId,source:e.tidalAlbumId?"TIDAL":e.qobuzAlbumId?"QOBUZ":void 0});return await f(`${t.apiUrl}/menu/album?${i}`,{method:"DELETE",credentials:"include",signal:n??null})}async function En(e,n){const t=d(),i=new g({albumId:e.tidalAlbumId?.toString()??e.qobuzAlbumId,source:e.tidalAlbumId?"TIDAL":e.qobuzAlbumId?"QOBUZ":void 0});return await f(`${t.apiUrl}/menu/album/re-favorite?${i}`,{method:"POST",credentials:"include",signal:n??null})}async function Cn(e,n){const t=d(),i=new g({taskId:`${e}`});return await f(`${t.apiUrl}/downloader/retry-download?${i}`,{method:"POST",credentials:"include",signal:n??null})}async function qn(e,n,t){const i=d(),s=new g({trackId:e.trackId?`${e.trackId}`:void 0,trackIds:e.trackIds?`${e.trackIds.join(",")}`:void 0,albumId:e.albumId?`${e.albumId}`:void 0,albumIds:e.albumIds?`${e.albumIds.join(",")}`:void 0,source:`${n}`});return await f(`${i.apiUrl}/downloader/download?${s}`,{method:"POST",credentials:"include",signal:t??null})}async function zn(e){const n=d();return await f(`${n.apiUrl}/downloader/download-tasks`,{credentials:"include",signal:e??null})}async function xn(e,n,t,i){const s=d(),r=new g({trackId:`${je(e)}`,max:`${Math.ceil(t)}`,source:`${n}`});return await f(`${s.apiUrl}/files/track/visualization?${r}`,{credentials:"include",signal:i??null})}async function Nn(e){const n=d(),t=new g({offset:"0",limit:"100"});return await f(`${n.apiUrl}/audio-zone?${t}`,{credentials:"include",signal:e??null})}async function Dn(e,n){const t=d(),i=new g({name:e});return await f(`${t.apiUrl}/audio-zone?${i}`,{method:"POST",credentials:"include",signal:n??null})}async function Qn(e,n){const t=d();return await f(`${t.apiUrl}/audio-zone`,{method:"PATCH",body:JSON.stringify(e),credentials:"include",signal:n??null})}async function Rn(e,n){const t=d(),i=new g({id:`${e}`});return await f(`${t.apiUrl}/audio-zone?${i}`,{method:"DELETE",credentials:"include",signal:n??null})}async function jn(e,n){const t=d(),i=new g({origins:`${e.join(",")}`});return await f(`${t.apiUrl}/scan/run-scan?${i}`,{method:"POST",credentials:"include",signal:n??null})}async function Fn(e,n){const t=d(),i=new g({origins:`${e.join(",")}`});return await f(`${t.apiUrl}/scan/start-scan?${i}`,{method:"POST",credentials:"include",signal:n??null})}async function Mn(e,n){const t=d(),i=new g({origin:`${e}`});return await f(`${t.apiUrl}/scan/scan-origins?${i}`,{method:"POST",credentials:"include",signal:n??null})}async function Bn(e,n){const t=d(),i=new g({path:`${e}`});return await f(`${t.apiUrl}/scan/scan-paths?${i}`,{method:"POST",credentials:"include",signal:n??null})}class de extends Error{constructor(n){let t=`Request failed: ${n.status}`;n.statusText&&(t+=` (${n.statusText})`),n.url&&(t+=` (url='${n.url}')`),typeof n.redirected<"u"&&(t+=` (redirected=${n.redirected})`),n.headers&&(t+=` (headers=${Pe(n.headers)})`),n.type&&(t+=` (type=${n.type})`),super(t),this.response=n}}async function f(e,n){const t=d();e[e.length-1]==="?"&&(e=e.substring(0,e.length-1));const i=new g,s=t.clientId;s&&i.set("clientId",s),i.size>0&&(e.indexOf("?")>0?e+=`&${i}`:e+=`?${i}`);const r=t.staticToken||t.token,o={"Content-Type":"application/json",...n?.headers??{}};r&&!o.Authorization&&(o.Authorization=r),n={...n,headers:o};const a=await fetch(e,n);if(!a.ok)throw new de(a);return await a.json()}function Yn(e){const n=new AbortController,t=n.signal;return{data:e(t),controller:n,signal:t}}const Y={};async function rt(e,n){const t=Y[e];t&&t.abort();const i=Yn(n);Y[e]=i.controller;let s;try{s=await i.data}catch(r){throw r}finally{delete Y[e]}return s}const b={getArtist:Be,getArtistCover:He,getArtistSourceCover:Ge,getAlbum:Je,getAlbums:W,getAllAlbums:Ve,getAlbumArtwork:Ye,getAlbumSourceArtwork:Ze,getAlbumTracks:We,getAlbumVersions:Xe,getTracks:Ke,getArtists:en,fetchSignatureToken:nn,refetchSignatureToken:on,validateSignatureTokenAndClient:rn,validateSignatureToken:an,magicToken:ln,globalSearch:ue,searchExternalMusicApi:D,searchAll:cn,getArtistFromTidalArtistId:un,getArtistFromQobuzArtistId:dn,getArtistFromTidalAlbumId:fn,getAlbumFromTidalAlbumId:yn,getAlbumFromQobuzAlbumId:bn,getTidalArtist:gn,getQobuzArtist:mn,getAllTidalArtistAlbums:pn,getAllQobuzArtistAlbums:$n,getTidalArtistAlbums:hn,getQobuzArtistAlbums:wn,getLibraryAlbumsFromTidalArtistId:An,getLibraryAlbumsFromQobuzArtistId:In,getTidalAlbum:vn,getQobuzAlbum:Sn,getTidalAlbumTracks:kn,getQobuzAlbumTracks:Tn,getYtAlbumTracks:On,getTidalTrack:Pn,getTrackUrlForSource:Un,addAlbumToLibrary:Ln,removeAlbumFromLibrary:_n,refavoriteAlbum:En,getDownloadTasks:zn,getTrackVisualization:xn,retryDownload:Cn,download:qn,getAudioZones:Nn,createAudioZone:Dn,updateAudioZone:Qn,deleteAudioZone:Rn,runScan:jn,startScan:Fn,enableScanOrigin:Mn,addScanPath:Bn};export{p as A,g as Q,b as a,F as b,Jn as c,H as d,it as e,Hn as f,st as g,Gn as h,Wn as i,Kn as j,te as k,et as l,d as m,le as n,rt as o,nt as p,Le as q,Vn as r,Fe as s,je as t,ke as u,tt as v,Pe as w,Xn as x};
//# sourceMappingURL=api.DZqBGp_p.js.map
