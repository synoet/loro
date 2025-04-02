type ItemWithId = { id: string | number; item: any };
type ItemWithIndex = { index: number; item: any };
type ItemWithIdAndIndex = { id: string | number; index: number; item: any };

export class VirtualMovableList {
  private list: ItemWithId[];

  constructor(list: Map<string | number, ItemWithIndex> = new Map()) {
    this.list = Array.from(
      list.entries(),
      ([id, { item }]) => ({ id, item })
    );
  }

  getById(id: string | number): ItemWithIdAndIndex | undefined {
    const index = this.list.findIndex((item) => item.id === id);

    if (index === -1) {
      return undefined;
    }

    return { id, index, item: this.list[index].item };
  }

  getByIndex(index: number): ItemWithId | undefined {
    if (index < 0 || index >= this.list.length) {
      return undefined;
    }
    return this.list[index];
  }

  move(fromIndex: number, toIndex: number): boolean {
    if (fromIndex < 0 || fromIndex >= this.list.length || 
        toIndex < 0 || toIndex >= this.list.length ||
        fromIndex === toIndex) {
      return false;
    }
    
    // Remove the element from the source position
    const [element] = this.list.splice(fromIndex, 1);
    
    // Insert it at the destination position
    this.list.splice(toIndex, 0, element);
    
    return true;
  }

  insert(index: number, item: any, id?: string | number): boolean {
    if (index < 0 || index > this.list.length) {
      return false;
    }
    
    const itemId = id ?? item.id;
    if (itemId === undefined) {
      return false;
    }
    
    this.list.splice(index, 0, { id: itemId, item });
    return true;
  }

  delete(index: number, count: number = 1): boolean {
    if (index < 0 || index >= this.list.length || count < 1) {
      return false;
    }
    
    count = Math.min(count, this.list.length - index);
    this.list.splice(index, count);
    return true;
  }
  
  getItems(): ItemWithIdAndIndex[] {
    return this.list.map((item, index) => ({
      id: item.id,
      index,
      item: item.item
    }));
  }
  
  size(): number {
    return this.list.length;
  }
  
  clear(): void {
    this.list = [];
  }
}