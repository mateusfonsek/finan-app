/**
 * Move o nó para o fim do `<body>`.
 *
 * Existe por causa de uma regra do CSS que morde silenciosamente: um ancestral
 * com `backdrop-filter` (ou `transform`, `filter`, `perspective`) vira o bloco
 * de contenção dos descendentes `position: fixed`. O cabeçalho de cada tela é
 * material translúcido — então um popover fixo declarado lá dentro passa a se
 * posicionar em relação AO CABEÇALHO, e as coordenadas calculadas a partir da
 * viewport saem deslocadas pela largura da barra lateral.
 *
 * Levar o nó pro `body` devolve a viewport como referência.
 */
export function portal(node: HTMLElement) {
  document.body.appendChild(node);
  return {
    destroy() {
      node.remove();
    },
  };
}
