import{s as w,j as b,k as D,m as L,o as M,p as j,q as B,r as k,t as H,H as W,v as q,w as T,x as m,y as R,z as Y,c as v,A as z,B as F,C as G,D as $,F as x,G as J,I as K,a as U,J as Q,e as X,h as S,K as Z}from"./i_b28a22.js";import{b as ee}from"./B01J20Qr.js";function le(e){return e.endsWith("capture")&&e!=="gotpointercapture"&&e!=="lostpointercapture"}const te=["beforeinput","click","change","dblclick","contextmenu","focusin","focusout","input","keydown","keyup","mousedown","mousemove","mouseout","mouseover","mouseup","pointerdown","pointermove","pointerout","pointerover","pointerup","touchend","touchmove","touchstart"];function ce(e){return te.includes(e)}const re={formnovalidate:"formNoValidate",ismap:"isMap",nomodule:"noModule",playsinline:"playsInline",readonly:"readOnly",defaultvalue:"defaultValue",defaultchecked:"defaultChecked",srcobject:"srcObject",novalidate:"noValidate",allowfullscreen:"allowFullscreen",disablepictureinpicture:"disablePictureInPicture",disableremoteplayback:"disableRemotePlayback"};function de(e){return e=e.toLowerCase(),re[e]??e}const ae=["touchstart","touchmove"];function oe(e){return ae.includes(e)}function ne(e){var t=D,a=L;w(null),b(null);try{return e()}finally{w(t),b(a)}}const O=new Set,A=new Set;function fe(e,t,a,s={}){function i(r){if(s.capture||y.call(t,r),!r.cancelBubble)return ne(()=>a==null?void 0:a.call(this,r))}return e.startsWith("pointer")||e.startsWith("touch")||e==="wheel"?B(()=>{t.addEventListener(e,i,s)}):t.addEventListener(e,i,s),i}function _e(e){for(var t=0;t<e.length;t++)O.add(e[t]);for(var a of A)a(e)}function y(e){var N;var t=this,a=t.ownerDocument,s=e.type,i=((N=e.composedPath)==null?void 0:N.call(e))||[],r=i[0]||e.target,c=0,_=e.__root;if(_){var d=i.indexOf(_);if(d!==-1&&(t===document||t===window)){e.__root=t;return}var p=i.indexOf(t);if(p===-1)return;d<=p&&(c=d)}if(r=i[c]||e.target,r!==t){M(e,"currentTarget",{configurable:!0,get(){return r||a}});var E=D,u=L;w(null),b(null);try{for(var o,n=[];r!==null;){var l=r.assignedSlot||r.parentNode||r.host||null;try{var f=r["__"+s];if(f!=null&&(!r.disabled||e.target===r))if(j(f)){var[P,...C]=f;P.apply(r,[e,...C])}else f.call(r,e)}catch(g){o?n.push(g):o=g}if(e.cancelBubble||l===t||l===null)break;r=l}if(o){for(let g of n)queueMicrotask(()=>{throw g});throw o}}finally{e.__root=t,delete e.currentTarget,w(E),b(u)}}}function pe(e,t){var a=t==null?"":typeof t=="object"?t+"":t;a!==(e.__t??(e.__t=e.nodeValue))&&(e.__t=a,e.nodeValue=a+"")}function ie(e,t){return V(e,t)}function he(e,t){k(),t.intro=t.intro??!1;const a=t.target,s=S,i=v;try{for(var r=H(a);r&&(r.nodeType!==8||r.data!==W);)r=q(r);if(!r)throw T;m(!0),R(r),Y();const c=V(e,{...t,anchor:r});if(v===null||v.nodeType!==8||v.data!==z)throw F(),T;return m(!1),c}catch(c){if(c===T)return t.recover===!1&&G(),k(),$(a),m(!1),ie(e,t);throw c}finally{m(s),R(i)}}const h=new Map;function V(e,{target:t,anchor:a,props:s={},events:i,context:r,intro:c=!0}){k();var _=new Set,d=u=>{for(var o=0;o<u.length;o++){var n=u[o];if(!_.has(n)){_.add(n);var l=oe(n);t.addEventListener(n,y,{passive:l});var f=h.get(n);f===void 0?(document.addEventListener(n,y,{passive:l}),h.set(n,1)):h.set(n,f+1)}}};d(x(O)),A.add(d);var p=void 0,E=J(()=>{var u=a??t.appendChild(K());return U(()=>{if(r){Q({});var o=X;o.c=r}i&&(s.$$events=i),S&&ee(u,null),p=e(u,s)||{},S&&(L.nodes_end=v),r&&Z()}),()=>{var l;for(var o of _){t.removeEventListener(o,y);var n=h.get(o);--n===0?(document.removeEventListener(o,y),h.delete(o)):h.set(o,n)}A.delete(d),u!==a&&((l=u.parentNode)==null||l.removeChild(u))}});return I.set(p,E),p}let I=new WeakMap;function ve(e,t){const a=I.get(e);return a?(I.delete(e),a(t)):Promise.resolve()}export{ce as a,fe as c,_e as d,he as h,le as i,ie as m,de as n,pe as s,ve as u};
